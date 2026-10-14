//! Enterprise Cross-Origin Resource Sharing (CORS) Middleware for Axum services.
//!
//! Fully driven by environment variables (`.env`):
//! - `CORS_ALLOWED_ORIGINS`: Comma-separated list of origins (e.g. "http://localhost:3000,http://localhost:8000")
//! - `CORS_ALLOWED_METHODS`: Comma-separated HTTP methods (e.g. "GET,POST,PUT,PATCH,DELETE,OPTIONS,HEAD")
//! - `CORS_ALLOWED_HEADERS`: Comma-separated request headers (e.g. "Authorization,Content-Type,Cookie")
//! - `CORS_EXPOSED_HEADERS`: Comma-separated response headers to expose (e.g. "Set-Cookie,Location")
//! - `CORS_ALLOW_CREDENTIALS`: Boolean flag ("true" / "false") for session cookies / credentials
//! - `CORS_MAX_AGE_SECS`: Preflight cache duration in seconds (e.g. 3600)

use axum::http::{HeaderName, HeaderValue, Method};
use std::env;
use std::time::Duration;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::{info, warn};

/// Configuration for the CORS middleware layer driven by `.env`.
#[derive(Clone, Debug, PartialEq)]
pub struct CorsConfig {
    /// Allowed origins loaded from `CORS_ALLOWED_ORIGINS`.
    pub allowed_origins: Vec<String>,
    /// Allowed HTTP methods loaded from `CORS_ALLOWED_METHODS`.
    pub allowed_methods: Vec<Method>,
    /// Allowed request headers loaded from `CORS_ALLOWED_HEADERS`.
    pub allowed_headers: Vec<HeaderName>,
    /// Exposed response headers loaded from `CORS_EXPOSED_HEADERS`.
    pub exposed_headers: Vec<HeaderName>,
    /// Whether to allow credentials (cookies, auth headers) loaded from `CORS_ALLOW_CREDENTIALS`.
    pub allow_credentials: bool,
    /// Preflight request cache duration in seconds loaded from `CORS_MAX_AGE_SECS`.
    pub max_age_secs: u64,
}

impl CorsConfig {
    /// Load CORS configuration strictly from `.env` and environment variables.
    ///
    /// Automatically ensures `.env` is loaded via `dotenvy::dotenv().ok()`.
    pub fn from_env() -> Self {
        // Ensure .env is loaded
        dotenvy::dotenv().ok();

        // 1. Allowed Origins (CORS_ALLOWED_ORIGINS)
        let allowed_origins: Vec<String> = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if allowed_origins.is_empty() {
            warn!(
                "CORS_ALLOWED_ORIGINS is not set in .env. No origins will be allowed by default!"
            );
        }

        // 2. Allowed Methods (CORS_ALLOWED_METHODS)
        let allowed_methods: Vec<Method> = env::var("CORS_ALLOWED_METHODS")
            .unwrap_or_else(|_| "GET,POST,PUT,PATCH,DELETE,OPTIONS,HEAD".to_string())
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .filter_map(|m| match m.parse::<Method>() {
                Ok(method) => Some(method),
                Err(err) => {
                    warn!(method = m, error = %err, "Invalid HTTP method in CORS_ALLOWED_METHODS ignored");
                    None
                }
            })
            .collect();

        // 3. Allowed Headers (CORS_ALLOWED_HEADERS)
        let allowed_headers: Vec<HeaderName> = env::var("CORS_ALLOWED_HEADERS")
            .unwrap_or_else(|_| {
                "Authorization,Content-Type,Accept,Cookie,Origin,X-Requested-With,X-Api-Key,X-Session-Token,X-Correlation-Id".to_string()
            })
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .filter_map(|h| match h.parse::<HeaderName>() {
                Ok(header) => Some(header),
                Err(err) => {
                    warn!(header = h, error = %err, "Invalid Header in CORS_ALLOWED_HEADERS ignored");
                    None
                }
            })
            .collect();

        // 4. Exposed Headers (CORS_EXPOSED_HEADERS)
        let exposed_headers: Vec<HeaderName> = env::var("CORS_EXPOSED_HEADERS")
            .unwrap_or_else(|_| "Set-Cookie,Location,Content-Disposition,X-Correlation-Id".to_string())
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .filter_map(|h| match h.parse::<HeaderName>() {
                Ok(header) => Some(header),
                Err(err) => {
                    warn!(header = h, error = %err, "Invalid Header in CORS_EXPOSED_HEADERS ignored");
                    None
                }
            })
            .collect();

        // 5. Allow Credentials (CORS_ALLOW_CREDENTIALS)
        let allow_credentials = env::var("CORS_ALLOW_CREDENTIALS")
            .map(|v| v.trim().eq_ignore_ascii_case("true") || v.trim() == "1")
            .unwrap_or(true);

        // 6. Max Age Seconds (CORS_MAX_AGE_SECS)
        let max_age_secs = env::var("CORS_MAX_AGE_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(3600);

        info!(
            origins = ?allowed_origins,
            allow_credentials,
            max_age_secs,
            "CORS configuration initialized from .env"
        );

        Self {
            allowed_origins,
            allowed_methods,
            allowed_headers,
            exposed_headers,
            allow_credentials,
            max_age_secs,
        }
    }

    /// Build the `tower_http::cors::CorsLayer` from this configuration.
    pub fn build(&self) -> CorsLayer {
        let mut layer = CorsLayer::new()
            .allow_methods(self.allowed_methods.clone())
            .allow_headers(self.allowed_headers.clone())
            .expose_headers(self.exposed_headers.clone())
            .max_age(Duration::from_secs(self.max_age_secs));

        if self.allow_credentials {
            layer = layer.allow_credentials(true);

            let header_values: Vec<HeaderValue> = self
                .allowed_origins
                .iter()
                .filter_map(|origin| match origin.parse::<HeaderValue>() {
                    Ok(val) => Some(val),
                    Err(err) => {
                        warn!(origin, error = %err, "Invalid origin header value ignored");
                        None
                    }
                })
                .collect();

            if header_values.is_empty() {
                warn!(
                    "No valid origins configured in CORS_ALLOWED_ORIGINS with allow_credentials=true"
                );
            } else {
                layer = layer.allow_origin(AllowOrigin::list(header_values));
            }
        } else if self.allowed_origins.contains(&"*".to_string()) {
            layer = layer.allow_origin(AllowOrigin::any());
        } else {
            let header_values: Vec<HeaderValue> = self
                .allowed_origins
                .iter()
                .filter_map(|origin| origin.parse::<HeaderValue>().ok())
                .collect();
            layer = layer.allow_origin(AllowOrigin::list(header_values));
        }

        layer
    }
}

/// Convenience function to build a `CorsLayer` directly from `.env`.
pub fn cors_layer_from_env() -> CorsLayer {
    CorsConfig::from_env().build()
}

/// Convenience function to build a `CorsLayer` from a specific `CorsConfig`.
pub fn cors_layer(config: &CorsConfig) -> CorsLayer {
    config.build()
}
