use super::http::HttpError;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("No active session found")]
    NoSession,

    #[error("Kratos API error: {0}")]
    ApiError(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<HttpError> for AuthError {
    fn from(err: HttpError) -> Self {
        match err {
            HttpError::Unauthorized => AuthError::NoSession,
            HttpError::ApiError { message, .. } => AuthError::ApiError(message),
            HttpError::Network(msg) => AuthError::Network(msg),
            HttpError::Serialization(msg) => AuthError::Serialization(msg),
            other => AuthError::ApiError(other.to_string()),
        }
    }
}

impl AuthError {
    /// If this error was generated from a Kratos 400 Bad Request,
    /// attempt to parse the updated `KratosLoginFlow` to access form validation errors.
    pub fn parse_kratos_flow(&self) -> Option<crate::core::auth::types::KratosLoginFlow> {
        match self {
            AuthError::ApiError(msg) => serde_json::from_str(msg).ok(),
            _ => None,
        }
    }
}
