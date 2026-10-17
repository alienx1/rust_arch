//! Ory Kratos Session Authentication Middleware for Axum.
//!
//! Validates `ory_kratos_session` cookie (or `X-Session-Token` header) by querying
//! Ory Kratos Public API (`/sessions/whoami`).
//! Optionally leverages `ValkeyCacheService` for sub-millisecond session caching.

use crate::services::cache::ValkeyCacheService;
use application_core::traits::{
    CacheService, IdentityError, IdentityProvisioningPort, IdentitySessionInfo, NewIdentity,
    RecoveryLink, SessionRevocationPort,
};
use async_trait::async_trait;
use axum::{
    Json,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Standard error response returned when session authentication fails.
#[derive(Debug, Serialize, Deserialize)]
pub struct KratosAuthErrorResponse {
    pub error: &'static str,
    pub message: String,
}

/// Authenticated user identity injected into request extensions upon successful verification.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    /// Ory Kratos Identity UUID (e.g. `019ca1ec-789a-7b2c-8d1e-000000000001`).
    pub identity_id: String,
    /// Ory Kratos Session UUID.
    pub session_id: String,
    /// User's primary email extracted from identity traits, if present.
    pub email: Option<String>,
    /// Raw traits object (e.g. name, role, metadata).
    pub traits: serde_json::Value,
}

impl AuthenticatedUser {
    /// Login name: the `username` trait, else the local part of the email
    pub fn username(&self) -> Option<String> {
        self.traits
            .get("username")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .or_else(|| self.email.as_deref().map(|e| e.split('@').next().unwrap_or(e).to_string()))
            .filter(|u| !u.trim().is_empty())
    }
}

/// Configuration for Ory Kratos session authentication middleware.
#[derive(Clone)]
pub struct KratosConfig {
    /// Base URL of the Ory Kratos Public API (e.g. `http://kratos.arch.svc:4433`).
    pub public_url: String,
    /// Base URL of the Ory Kratos Admin API (e.g. `http://kratos.arch.svc:4434`).
    pub admin_url: String,
    /// HTTP client used to contact Kratos.
    pub client: Client,
    /// Optional Valkey cache service for session caching.
    pub cache: Option<ValkeyCacheService>,
    /// Cache TTL in seconds (default: 60s).
    pub cache_ttl_secs: u64,
}

impl KratosConfig {
    /// Create a new Kratos configuration with the given public API URL.
    pub fn new(public_url: impl Into<String>) -> Self {
        let pub_url = public_url.into().trim_end_matches('/').to_string();
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            admin_url: pub_url.replace(":4433", ":4434"),
            public_url: pub_url,
            client,
            cache: None,
            cache_ttl_secs: 60,
        }
    }

    /// Set custom Admin API URL.
    pub fn with_admin_url(mut self, admin_url: impl Into<String>) -> Self {
        self.admin_url = admin_url.into().trim_end_matches('/').to_string();
        self
    }

    /// Enable session caching using Valkey.
    pub fn with_cache(mut self, cache: ValkeyCacheService) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Set session cache TTL in seconds (default is 60).
    pub fn with_cache_ttl(mut self, ttl_secs: u64) -> Self {
        self.cache_ttl_secs = ttl_secs;
        self
    }

    /// Use a custom HTTP client.
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    fn session_index_key(session_id: &str) -> String {
        format!("kratos_session:idx:{session_id}")
    }

    /// Remember which cache keys hold `session_id`, so revocation can evict them
    /// (cache keys are derived from the cookie/token, which revocation does not know).
    async fn index_cache_key(&self, session_id: &str, cache_key: &str) {
        let Some(cache) = &self.cache else { return };
        if session_id.is_empty() {
            return;
        }
        let idx = Self::session_index_key(session_id);
        let mut keys: Vec<String> = cache.get_json(&idx).await.ok().flatten().unwrap_or_default();
        if !keys.iter().any(|k| k == cache_key) {
            keys.push(cache_key.to_string());
        }
        // Keep the index at least as long as the cached entry it points to.
        let _ = cache.set_json(&idx, &keys, Some(self.cache_ttl_secs.saturating_add(60))).await;
    }

    /// Resolve a Kratos identity id, or a login identifier (email or username), to the Kratos
    /// identity id. A UUID is taken as the id itself; anything else is looked up as a credential
    /// identifier (both `email` and `username` are identifiers in the identity schema).
    async fn resolve_identity_id(&self, user_ref: &str) -> Result<Option<String>, String> {
        if uuid::Uuid::parse_str(user_ref).is_ok() {
            return Ok(Some(user_ref.to_string()));
        }
        self.lookup_identity_id(user_ref).await
    }

    /// Kratos identity id of the identity that has `identifier` (email / username) as a
    /// credential identifier; `None` when there is none. Never treats the input as an id.
    async fn lookup_identity_id(&self, identifier: &str) -> Result<Option<String>, String> {
        Ok(self.lookup_identity(identifier).await?.and_then(|i| i["id"].as_str().map(ToString::to_string)))
    }

    /// The identity that has `identifier` as a credential identifier (the raw Kratos object)
    async fn lookup_identity(&self, identifier: &str) -> Result<Option<serde_json::Value>, String> {
        let res = self
            .client
            .get(format!(
                "{}/admin/identities?credentials_identifier={}",
                self.admin_url,
                identifier.replace('%', "%25").replace('+', "%2B").replace('&', "%26").replace('#', "%23")
            ))
            .send()
            .await
            .map_err(|e| format!("Kratos Admin API request failed: {e}"))?;
        if !res.status().is_success() {
            return Err(format!("Kratos Admin API identity lookup failed with status: {}", res.status()));
        }
        let found: Vec<serde_json::Value> = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse Kratos identities response: {e}"))?;
        Ok(found.into_iter().next())
    }

    /// Evict every cached verification result belonging to `session_id`.
    async fn evict_session_cache(&self, session_id: &str) {
        let Some(cache) = &self.cache else { return };
        let idx = Self::session_index_key(session_id);
        let keys: Vec<String> = cache.get_json(&idx).await.ok().flatten().unwrap_or_default();
        for k in keys {
            let _ = cache.del(&k).await;
        }
        let _ = cache.del(&idx).await;
    }

    /// Verify an Ory Kratos session given an optional cookie and/or session token.
    pub async fn verify_session(
        &self,
        cookie_header: Option<&str>,
        session_token: Option<&str>,
    ) -> Result<AuthenticatedUser, String> {
        // Build a cache key if caching is enabled
        let cache_key = if self.cache.is_some() {
            if let Some(token) = session_token {
                Some(format!("kratos_session:token:{}", token))
            } else if let Some(cookie) = cookie_header {
                Some(format!("kratos_session:cookie:{}", cookie))
            } else {
                None
            }
        } else {
            None
        };

        // 1. Check Valkey Cache
        if let (Some(cache), Some(key)) = (&self.cache, &cache_key) {
            if let Ok(Some(cached_user)) = cache.get_json::<AuthenticatedUser>(key).await {
                return Ok(cached_user);
            }
        }

        // 2. Call Ory Kratos /sessions/whoami
        let url = format!("{}/sessions/whoami", self.public_url);
        let mut req = self.client.get(&url);

        if let Some(token) = session_token {
            req = req.header("X-Session-Token", token);
        } else if let Some(cookie) = cookie_header {
            req = req.header(header::COOKIE, cookie);
        } else {
            return Err("No session credentials provided".to_string());
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ory Kratos: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            return Err(format!("Ory Kratos rejected session with status: {status}"));
        }

        let session_json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Kratos session response: {e}"))?;

        // Check if session is active
        let is_active = session_json["active"].as_bool().unwrap_or(false);
        if !is_active {
            return Err("Session is inactive or expired".to_string());
        }

        let identity_id = session_json["identity"]["id"]
            .as_str()
            .ok_or_else(|| "Missing identity.id in Kratos session".to_string())?
            .to_string();

        let session_id = session_json["id"].as_str().unwrap_or_default().to_string();

        let traits = session_json["identity"]["traits"].clone();
        let email = traits["email"].as_str().map(ToString::to_string);

        let authenticated_user = AuthenticatedUser {
            identity_id,
            session_id,
            email,
            traits,
        };

        // 3. Store in Valkey Cache if enabled
        if let (Some(cache), Some(key)) = (&self.cache, &cache_key) {
            let _ = cache
                .set_json(key, &authenticated_user, Some(self.cache_ttl_secs))
                .await;
            self.index_cache_key(&authenticated_user.session_id, key).await;
        }

        Ok(authenticated_user)
    }
}

/// Axum middleware enforcing Ory Kratos session authentication.
///
/// Returns 401 Unauthorized if:
/// - Neither `ory_kratos_session` cookie nor `X-Session-Token` header is present.
/// - The session is invalid, expired, or inactive.
///
/// On success, injects `AuthenticatedUser` into `request.extensions_mut()` and, as a clone, into the
/// extensions of the returned response (for outer layers such as the audit trail).
pub async fn require_kratos_session(
    State(config): State<KratosConfig>,
    mut request: Request,
    next: Next,
) -> Response {
    let headers = request.headers();

    // 1. Extract X-Session-Token header (preferred for API/mobile clients)
    let session_token = headers
        .get("x-session-token")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string);

    // 2. Extract ory_kratos_session cookie (preferred for Web browsers)
    let session_cookie = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .map(str::trim)
                .find(|c| c.starts_with("ory_kratos_session="))
        })
        .map(ToString::to_string);

    if session_token.is_none() && session_cookie.is_none() {
        let error = KratosAuthErrorResponse {
            error: "UNAUTHORIZED",
            message: "Missing 'ory_kratos_session' cookie or 'X-Session-Token' header".to_string(),
        };
        return (StatusCode::UNAUTHORIZED, Json(error)).into_response();
    }

    match config
        .verify_session(session_cookie.as_deref(), session_token.as_deref())
        .await
    {
        Ok(user) => {
            request.extensions_mut().insert(user.clone());
            let mut response = next.run(request).await;
            // Outer middlewares (audit trail) cannot see request extensions: expose who it was
            response.extensions_mut().insert(user);
            response
        }
        Err(err_msg) => {
            let error = KratosAuthErrorResponse {
                error: "UNAUTHORIZED",
                message: err_msg,
            };
            (StatusCode::UNAUTHORIZED, Json(error)).into_response()
        }
    }
}

#[async_trait]
impl SessionRevocationPort for KratosConfig {
    async fn revoke_session(&self, session_id: &str) -> Result<bool, String> {
        let url = format!("{}/admin/sessions/{}", self.admin_url, session_id);
        tracing::info!("Revoking session via Ory Kratos Admin API: {url}");
        let res = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| format!("Kratos Admin API request failed: {e}"))?;

        // Evict regardless of the Kratos result: a stale cached "valid" entry is never wanted here.
        self.evict_session_cache(session_id).await;

        let status = res.status();
        if status.is_success() {
            Ok(true)
        } else {
            Err(format!("Kratos Admin API rejected session revoke with status: {status}"))
        }
    }

    async fn revoke_user_sessions(&self, user_ref: &str) -> Result<bool, String> {
        let Some(user_id) = self.resolve_identity_id(user_ref).await? else {
            // No Kratos identity => nothing to revoke.
            return Ok(true);
        };
        let user_id = user_id.as_str();
        // Collect session ids first: after the bulk delete Kratos can no longer list them.
        let sessions = self.list_user_sessions(user_id).await.unwrap_or_default();

        let url = format!("{}/admin/identities/{}/sessions", self.admin_url, user_id);
        tracing::info!("Revoking user sessions via Ory Kratos Admin API: {url}");
        let res = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| format!("Kratos Admin API request failed: {e}"))?;

        for s in &sessions {
            self.evict_session_cache(&s.session_id).await;
        }

        let status = res.status();
        // 404: identity unknown to Kratos => nothing left to revoke.
        if status.is_success() || status == StatusCode::NOT_FOUND {
            Ok(true)
        } else {
            Err(format!("Kratos Admin API rejected user session revoke with status: {status}"))
        }
    }

    async fn set_user_active(&self, user_ref: &str, active: bool) -> Result<bool, String> {
        let Some(identity_id) = self.resolve_identity_id(user_ref).await? else {
            return Ok(false);
        };
        let url = format!("{}/admin/identities/{}", self.admin_url, identity_id);
        let state = if active { "active" } else { "inactive" };
        tracing::info!("Setting Kratos identity {identity_id} state={state}");
        let patch = serde_json::json!([{ "op": "replace", "path": "/state", "value": state }]);
        let res = self
            .client
            .patch(&url)
            .json(&patch)
            .send()
            .await
            .map_err(|e| format!("Kratos Admin API request failed: {e}"))?;

        let status = res.status();
        if !status.is_success() {
            return Err(format!("Kratos Admin API rejected identity state change with status: {status}"));
        }
        if !active {
            // An inactive identity must not keep working sessions or cached verifications.
            self.revoke_user_sessions(&identity_id).await?;
        }
        Ok(true)
    }

    async fn list_user_sessions(&self, user_ref: &str) -> Result<Vec<IdentitySessionInfo>, String> {
        let Some(user_id) = self.resolve_identity_id(user_ref).await? else {
            return Ok(Vec::new());
        };
        let user_id = user_id.as_str();
        let url = format!("{}/admin/identities/{}/sessions?active=true", self.admin_url, user_id);
        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Kratos Admin API request failed: {e}"))?;

        let status = res.status();
        if status == StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }
        if !status.is_success() {
            return Err(format!("Kratos Admin API list sessions failed with status: {status}"));
        }

        let raw: Vec<serde_json::Value> = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse Kratos sessions response: {e}"))?;

        let text = |v: &serde_json::Value| v.as_str().filter(|s| !s.is_empty()).map(ToString::to_string);
        Ok(raw
            .into_iter()
            .filter(|s| s["active"].as_bool().unwrap_or(true))
            .filter_map(|s| {
                let device = &s["devices"][0];
                Some(IdentitySessionInfo {
                    session_id: text(&s["id"])?,
                    user_agent: text(&device["user_agent"]),
                    ip_address: text(&device["ip_address"]),
                    location: text(&device["location"]),
                    authenticated_at: text(&s["authenticated_at"]),
                })
            })
            .collect())
    }
}

#[async_trait]
impl IdentityProvisioningPort for KratosConfig {
    async fn create_identity(&self, identity: NewIdentity) -> Result<String, IdentityError> {
        let url = format!("{}/admin/identities", self.admin_url);
        // No credentials: the account sets its password through a recovery link.
        let body = serde_json::json!({
            "schema_id": "default",
            "state": "active",
            "traits": {
                "email": identity.email,
                "username": identity.username,
                "name": { "first": identity.first_name, "last": identity.last_name },
            },
        });
        let res = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| IdentityError::Provider(format!("Kratos Admin API request failed: {e}")))?;
        let status = res.status();
        if status == StatusCode::CONFLICT {
            return Err(IdentityError::Conflict(format!(
                "{} / {}",
                identity.email, identity.username
            )));
        }
        if !status.is_success() {
            let detail = res.text().await.unwrap_or_default();
            return Err(IdentityError::Provider(format!(
                "Kratos Admin API rejected identity creation ({status}): {detail}"
            )));
        }
        let created: serde_json::Value = res
            .json()
            .await
            .map_err(|e| IdentityError::Provider(format!("Failed to parse Kratos response: {e}")))?;
        created["id"]
            .as_str()
            .map(ToString::to_string)
            .ok_or_else(|| IdentityError::Provider("Kratos response had no identity id".to_string()))
    }

    async fn delete_identity(&self, identity_id: &str) -> Result<(), IdentityError> {
        let res = self
            .client
            .delete(format!("{}/admin/identities/{identity_id}", self.admin_url))
            .send()
            .await
            .map_err(|e| IdentityError::Provider(format!("Kratos Admin API request failed: {e}")))?;
        let status = res.status();
        if status.is_success() || status == StatusCode::NOT_FOUND {
            Ok(())
        } else {
            Err(IdentityError::Provider(format!(
                "Kratos Admin API rejected identity deletion ({status})"
            )))
        }
    }

    async fn create_recovery_link(&self, identifier: &str) -> Result<RecoveryLink, IdentityError> {
        let identity_id = self
            .find_identity_id(identifier)
            .await?
            .ok_or_else(|| IdentityError::NotFound(identifier.to_string()))?;
        let res = self
            .client
            .post(format!("{}/admin/recovery/link", self.admin_url))
            .json(&serde_json::json!({ "identity_id": identity_id, "expires_in": "24h" }))
            .send()
            .await
            .map_err(|e| IdentityError::Provider(format!("Kratos Admin API request failed: {e}")))?;
        let status = res.status();
        if !status.is_success() {
            let detail = res.text().await.unwrap_or_default();
            return Err(IdentityError::Provider(format!(
                "Kratos Admin API rejected recovery link ({status}): {detail}"
            )));
        }
        let body: serde_json::Value = res
            .json()
            .await
            .map_err(|e| IdentityError::Provider(format!("Failed to parse Kratos response: {e}")))?;
        let url = body["recovery_link"]
            .as_str()
            .ok_or_else(|| IdentityError::Provider("Kratos response had no recovery link".to_string()))?
            .to_string();
        Ok(RecoveryLink { url, expires_at: body["expires_at"].as_str().map(ToString::to_string) })
    }

    async fn find_identity_id(&self, identifier: &str) -> Result<Option<String>, IdentityError> {
        self.lookup_identity_id(identifier).await.map_err(IdentityError::Provider)
    }

    async fn find_username(&self, identifier: &str) -> Result<Option<String>, IdentityError> {
        let identity = self.lookup_identity(identifier).await.map_err(IdentityError::Provider)?;
        Ok(identity.and_then(|i| i["traits"]["username"].as_str().map(ToString::to_string)))
    }

    async fn update_email(&self, login: &str, new_email: &str) -> Result<(), IdentityError> {
        let Some(identity) = self.lookup_identity(login).await.map_err(IdentityError::Provider)? else {
            return Err(IdentityError::NotFound(login.to_string()));
        };
        let Some(id) = identity["id"].as_str() else {
            return Err(IdentityError::Provider("identity without id".into()));
        };
        if identity["traits"]["email"].as_str().is_some_and(|e| e.eq_ignore_ascii_case(new_email)) {
            return Ok(());
        }
        let patch = serde_json::json!([{ "op": "replace", "path": "/traits/email", "value": new_email }]);
        let res = self
            .client
            .patch(format!("{}/admin/identities/{id}", self.admin_url))
            .json(&patch)
            .send()
            .await
            .map_err(|e| IdentityError::Provider(format!("Kratos Admin API request failed: {e}")))?;
        match res.status() {
            s if s.is_success() => Ok(()),
            reqwest::StatusCode::CONFLICT => Err(IdentityError::Conflict(new_email.to_string())),
            s => Err(IdentityError::Provider(format!("Kratos Admin API rejected the e-mail change with status: {s}"))),
        }
    }

    async fn find_email(&self, identifier: &str) -> Result<Option<String>, IdentityError> {
        let identity = self.lookup_identity(identifier).await.map_err(IdentityError::Provider)?;
        Ok(identity.and_then(|i| i["traits"]["email"].as_str().map(ToString::to_string)))
    }
}
