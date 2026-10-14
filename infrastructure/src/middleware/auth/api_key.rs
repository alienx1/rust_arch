//! API Key authentication middleware for Service-to-Service and Report endpoints.

use axum::{
    Json,
    extract::{Request, State},
    http::{HeaderMap, HeaderName, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use subtle::ConstantTimeEq;

/// Standard error response returned when API Key validation fails.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthErrorResponse {
    pub error: &'static str,
    pub message: String,
}

/// Extension injected into request after successful API Key verification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthenticatedApiKey {
    pub key: String,
    pub client_name: Option<String>,
}

/// Configuration for API Key verification middleware.
#[derive(Clone, Debug)]
pub struct ApiKeyConfig {
    /// Expected header name (defaults to `x-api-key`).
    pub header_name: HeaderName,
    /// Set of authorized API keys.
    pub valid_keys: Arc<HashSet<String>>,
    /// Whether to also inspect `Authorization: Bearer <key>` or `Authorization: ApiKey <key>`.
    pub allow_authorization_header: bool,
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            header_name: HeaderName::from_static("x-api-key"),
            valid_keys: Arc::new(HashSet::new()),
            allow_authorization_header: true,
        }
    }
}

impl ApiKeyConfig {
    /// Create new config with a single valid API key.
    pub fn new(single_key: impl Into<String>) -> Self {
        let mut keys = HashSet::new();
        keys.insert(single_key.into());
        Self {
            header_name: HeaderName::from_static("x-api-key"),
            valid_keys: Arc::new(keys),
            allow_authorization_header: true,
        }
    }

    /// Create new config with multiple valid API keys (e.g. for multiple clients/services).
    pub fn from_keys(keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let set: HashSet<String> = keys.into_iter().map(Into::into).collect();
        Self {
            header_name: HeaderName::from_static("x-api-key"),
            valid_keys: Arc::new(set),
            allow_authorization_header: true,
        }
    }

    /// Set a custom header name (e.g. `x-report-api-key`).
    pub fn with_header(mut self, header_name: HeaderName) -> Self {
        self.header_name = header_name;
        self
    }

    /// Set whether Authorization header is accepted as fallback.
    pub fn with_authorization_header(mut self, allow: bool) -> Self {
        self.allow_authorization_header = allow;
        self
    }

    /// Verify an incoming key using constant-time comparison against all valid keys.
    pub fn verify(&self, candidate: &str) -> bool {
        let candidate_bytes = candidate.as_bytes();
        let mut matched = false;

        for valid_key in self.valid_keys.iter() {
            let valid_bytes = valid_key.as_bytes();
            if valid_bytes.len() == candidate_bytes.len() {
                if valid_bytes.ct_eq(candidate_bytes).into() {
                    matched = true;
                }
            }
        }

        matched
    }

    /// Extract API key from request headers.
    pub fn extract_key<'a>(&self, headers: &'a HeaderMap) -> Option<&'a str> {
        // 1. Try specified header (e.g. x-api-key)
        if let Some(val) = headers.get(&self.header_name) {
            if let Ok(str_val) = val.to_str() {
                let trimmed = str_val.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed);
                }
            }
        }

        // 2. Try Authorization header if allowed (Bearer <key> or ApiKey <key>)
        if self.allow_authorization_header {
            if let Some(auth_val) = headers.get(header::AUTHORIZATION) {
                if let Ok(auth_str) = auth_val.to_str() {
                    let trimmed = auth_str.trim();
                    if let Some(token) = trimmed.strip_prefix("Bearer ") {
                        let token = token.trim();
                        if !token.is_empty() {
                            return Some(token);
                        }
                    } else if let Some(token) = trimmed.strip_prefix("ApiKey ") {
                        let token = token.trim();
                        if !token.is_empty() {
                            return Some(token);
                        }
                    }
                }
            }
        }

        None
    }
}

/// Axum middleware function to enforce API Key authentication.
///
/// Can be plugged into an Axum router using:
/// ```rust,ignore
/// use axum::middleware;
/// use infrastructure::middleware::auth::api_key::{require_api_key, ApiKeyConfig};
///
/// let config = ApiKeyConfig::new("my-secret-report-key");
/// let router = Router::new()
///     .route("/api/v1/reports", post(generate_report))
///     .route_layer(middleware::from_fn_with_state(config, require_api_key));
/// ```
pub async fn require_api_key(
    State(config): State<ApiKeyConfig>,
    mut request: Request,
    next: Next,
) -> Response {
    let candidate_key = match config.extract_key(request.headers()) {
        Some(key) => key.to_string(),
        None => {
            let error = AuthErrorResponse {
                error: "UNAUTHORIZED",
                message: format!(
                    "Missing API key. Provide '{}' header or 'Authorization: Bearer <key>'",
                    config.header_name.as_str()
                ),
            };
            return (StatusCode::UNAUTHORIZED, Json(error)).into_response();
        }
    };

    if !config.verify(&candidate_key) {
        let error = AuthErrorResponse {
            error: "UNAUTHORIZED",
            message: "Invalid API key".to_string(),
        };
        return (StatusCode::UNAUTHORIZED, Json(error)).into_response();
    }

    // Insert AuthenticatedApiKey into request extensions
    request.extensions_mut().insert(AuthenticatedApiKey {
        key: candidate_key,
        client_name: None,
    });

    next.run(request).await
}
