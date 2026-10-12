use super::types::{KratosLoginFlow, KratosLogoutFlow, KratosSession, LoginPayload};
use crate::core::errors::{AuthError, HttpError};
use crate::core::requestor::HttpClient;

#[derive(Clone, Debug)]
pub struct KratosClient {
    http: HttpClient,
    kratos_url: String,
}

impl KratosClient {
    pub fn new(kratos_url: impl Into<String>) -> Self {
        let base_url = kratos_url.into();
        Self {
            http: HttpClient::new(&base_url),
            kratos_url: base_url,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.kratos_url
    }

    /// Check if the user currently has an active Kratos session.
    ///
    /// Sends `GET /sessions/whoami` with credentials (cookies) included.
    /// Returns `Ok(None)` if no session cookie exists or session is expired.
    pub async fn whoami(&self) -> Result<Option<KratosSession>, AuthError> {
        let res = self.http.get("/sessions/whoami").send().await;

        match res {
            Ok(resp) => {
                let session: KratosSession = resp.json()?;
                if session.active {
                    Ok(Some(session))
                } else {
                    Ok(None)
                }
            }
            Err(HttpError::Unauthorized) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Fetch all active sessions for the currently authenticated identity.
    ///
    /// Sends `GET /sessions` with credentials (cookies) included.
    pub async fn list_my_sessions(&self) -> Result<Vec<KratosSession>, AuthError> {
        let res = self.http.get("/sessions").send().await;

        match res {
            Ok(resp) => {
                let sessions: Vec<KratosSession> = resp.json()?;
                Ok(sessions.into_iter().filter(|s| s.active).collect())
            }
            Err(HttpError::Unauthorized) => Ok(Vec::new()),
            Err(e) => Err(e.into()),
        }
    }

    /// Revoke all active sessions for the current identity except the active one.
    ///
    /// Sends `DELETE /sessions` with credentials included.
    pub async fn disable_my_other_sessions(&self) -> Result<(), AuthError> {
        let _ = self.http.delete("/sessions").send().await?;
        Ok(())
    }

    /// Revoke a specific active session by ID.
    ///
    /// Sends `DELETE /sessions/{id}` with credentials included.
    pub async fn disable_my_session(&self, session_id: &str) -> Result<(), AuthError> {
        let _ = self.http.delete(&format!("/sessions/{session_id}")).send().await?;
        Ok(())
    }

    /// Initialize a new self-service browser login flow.
    ///
    /// Sends `GET /self-service/login/browser`.
    pub async fn init_login_flow(&self) -> Result<KratosLoginFlow, AuthError> {
        let resp = self
            .http
            .get("/self-service/login/browser")
            .header("Accept", "application/json")
            .send()
            .await?;

        let flow: KratosLoginFlow = resp.json()?;
        Ok(flow)
    }

    /// Fetch existing self-service login flow by flow ID.
    ///
    /// Sends `GET /self-service/login/flows?id={flow_id}`.
    pub async fn get_login_flow(&self, flow_id: &str) -> Result<KratosLoginFlow, AuthError> {
        let resp = self
            .http
            .get(&format!("/self-service/login/flows?id={flow_id}"))
            .header("Accept", "application/json")
            .send()
            .await?;

        let flow: KratosLoginFlow = resp.json()?;
        Ok(flow)
    }

    /// Submit credentials to Kratos login flow action URL.
    ///
    /// Sends `POST {action_url}` with JSON body and cookie credentials included.
    pub async fn submit_login(
        &self,
        action_url: &str,
        payload: &LoginPayload,
    ) -> Result<KratosSession, AuthError> {
        let clean_path = extract_path_and_query(action_url);
        let resp = self
            .http
            .post(clean_path)
            .header("Accept", "application/json")
            .json(payload)?
            .send()
            .await?;

        // On successful login, Kratos returns the active session (or successful flow with session)
        if let Ok(session) = resp.json::<KratosSession>() {
            return Ok(session);
        }

        // If returned flow object wrapping session
        if let Ok(wrapper) = resp.json::<serde_json::Value>() {
            if let Some(session_val) = wrapper.get("session") {
                let session: KratosSession = serde_json::from_value(session_val.clone())
                    .map_err(|e| AuthError::Serialization(e.to_string()))?;
                return Ok(session);
            }
        }

        // Fallback: verify session via whoami
        self.whoami().await?.ok_or_else(|| {
            AuthError::ApiError("Login completed but session is inactive".to_string())
        })
    }

    /// Initialize a new self-service browser settings flow (needs a session cookie).
    ///
    /// Sends `GET /self-service/settings/browser`.
    pub async fn init_settings_flow(&self) -> Result<KratosLoginFlow, AuthError> {
        let resp = self
            .http
            .get("/self-service/settings/browser")
            .header("Accept", "application/json")
            .send()
            .await?;
        Ok(resp.json()?)
    }

    /// Fetch an existing settings flow, e.g. the one a recovery link redirects to.
    ///
    /// Sends `GET /self-service/settings/flows?id={flow_id}`.
    pub async fn get_settings_flow(&self, flow_id: &str) -> Result<KratosLoginFlow, AuthError> {
        let resp = self
            .http
            .get(&format!("/self-service/settings/flows?id={flow_id}"))
            .header("Accept", "application/json")
            .send()
            .await?;
        Ok(resp.json()?)
    }

    /// Set a new password through a settings flow.
    ///
    /// Sends `POST {action_url}`; a 400 answer carries the flow with field errors
    /// (see [`AuthError::parse_kratos_flow`]).
    pub async fn submit_settings_password(
        &self,
        action_url: &str,
        password: &str,
        csrf_token: Option<String>,
    ) -> Result<(), AuthError> {
        let mut body = serde_json::json!({ "method": "password", "password": password });
        if let Some(token) = csrf_token {
            body["csrf_token"] = serde_json::Value::String(token);
        }
        let _ = self
            .http
            .post(extract_path_and_query(action_url))
            .header("Accept", "application/json")
            .json(&body)?
            .send()
            .await?;
        Ok(())
    }

    /// Initialize self-service browser logout flow.
    ///
    /// Sends `GET /self-service/logout/browser`.
    pub async fn init_logout_flow(&self) -> Result<KratosLogoutFlow, AuthError> {
        let resp = self
            .http
            .get("/self-service/logout/browser")
            .header("Accept", "application/json")
            .send()
            .await?;

        let flow: KratosLogoutFlow = resp.json()?;
        Ok(flow)
    }

    /// Perform logout by fetching logout flow and invoking the logout URL.
    pub async fn logout(&self) -> Result<(), AuthError> {
        let flow = self.init_logout_flow().await?;
        let clean_path = extract_path_and_query(&flow.logout_url);
        let _ = self.http.get(clean_path).send().await?;
        Ok(())
    }
}

/// Helper to extract relative path and query from an absolute URL or path.
///
/// This ensures requests always route through the configured `HttpClient` (supporting
/// relative base URLs, reverse proxies like `/api/kratos`, or direct custom endpoints).
fn extract_path_and_query(url_or_path: &str) -> &str {
    if let Some(idx) = url_or_path.find("://") {
        let after_scheme = &url_or_path[idx + 3..];
        if let Some(path_idx) = after_scheme.find('/') {
            return &after_scheme[path_idx..];
        }
        return "/";
    }
    url_or_path
}
