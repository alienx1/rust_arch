//! Who is calling: service API key and Ory Kratos session verification.

pub mod api_key;
pub mod kratos;

pub use api_key::{ApiKeyConfig, AuthenticatedApiKey, require_api_key};
pub use kratos::{AuthenticatedUser, KratosConfig, require_kratos_session};
