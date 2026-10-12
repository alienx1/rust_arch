use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Cache connection error: {0}")]
    ConnectionError(String),

    #[error("Cache operation failed: {0}")]
    OperationFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}
