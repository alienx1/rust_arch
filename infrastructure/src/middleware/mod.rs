//! Infrastructure middlewares, grouped by what they do:
//!
//! | Group | Contents |
//! |-------|----------|
//! | [`auth`] | `require_api_key` (service-to-service), `require_kratos_session` (users) |
//! | [`audit`] | `audit_trail` and its body/target helpers |
//! | [`cache`] | `invalidate_cache_on_write` (Valkey read cache) |
//! | [`http`] | CORS, request language, localized error bodies |
//!
//! Every item is re-exported here, so `infrastructure::middleware::AuthenticatedUser` and the like
//! keep working; reach into a group only for its submodules (e.g. `middleware::audit::audit_body`).

pub mod audit;
pub mod auth;
pub mod cache;
pub mod http;

pub use audit::{AuditTrailState, audit_trail, derive_audit_target, should_audit};
pub use auth::{ApiKeyConfig, AuthenticatedApiKey, AuthenticatedUser, KratosConfig, require_api_key, require_kratos_session};
pub use cache::{CacheInvalidation, InvalidationRule, invalidate_cache_on_write, namespaces_for};
pub use http::{
    CorsConfig, ErrorLocalizer, cors_layer, cors_layer_from_env, localize_error_responses, set_request_language,
};
