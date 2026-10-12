use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("HTTP error status {status}: {message}")]
    ApiError { status: u16, message: String },

    #[error("Unauthorized: session is missing or expired")]
    Unauthorized,

    #[error("Forbidden: access denied")]
    Forbidden,

    #[error("Resource not found: {0}")]
    NotFound(String),
}

impl From<gloo_net::Error> for HttpError {
    fn from(err: gloo_net::Error) -> Self {
        HttpError::Network(err.to_string())
    }
}
