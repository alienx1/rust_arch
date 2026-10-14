//! HTTP plumbing: CORS, request language and localized error bodies.

pub mod cors;
pub mod error_localization;
pub mod request_language;

pub use cors::{CorsConfig, cors_layer, cors_layer_from_env};
pub use error_localization::{ErrorLocalizer, localize_error_responses};
pub use request_language::set_request_language;
