use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventError {
    #[error("Failed to publish event to topic '{topic}': {reason}")]
    PublishFailed { topic: String, reason: String },

    #[error("Event serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Event streaming connection error: {0}")]
    ConnectionError(String),
}
