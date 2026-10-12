//! Infrastructure middlewares: API key, cache invalidation, CORS, Ory Kratos session verification, audit trail,
//! error localization and request language.

pub mod api_key;
pub mod audit_body;
pub mod audit_target;
pub mod audit_trail;
pub mod cache_invalidation;
pub mod cors;
pub mod error_localization;
pub mod kratos;
pub mod request_language;

pub use api_key::{ApiKeyConfig, AuthenticatedApiKey, require_api_key};
pub use audit_target::{derive_audit_target, should_audit};
pub use audit_trail::{AuditTrailState, audit_trail};
pub use cache_invalidation::{CacheInvalidation, InvalidationRule, invalidate_cache_on_write, namespaces_for};
pub use cors::{CorsConfig, cors_layer, cors_layer_from_env};
pub use error_localization::{ErrorLocalizer, localize_error_responses};
pub use kratos::{AuthenticatedUser, KratosConfig, require_kratos_session};
pub use request_language::set_request_language;
