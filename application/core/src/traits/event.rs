use crate::errors::EventError;
use async_trait::async_trait;
use serde::Serialize;

#[async_trait]
pub trait EventService: Send + Sync {
    /// Publish raw event bytes to a Kafka/Redpanda topic.
    async fn publish(
        &self,
        topic: &str,
        key: Option<&str>,
        payload: &[u8],
    ) -> Result<(), EventError>;

    /// Publish a serialized JSON event to a Kafka/Redpanda topic.
    async fn publish_json<T: Serialize + Sync + Send>(
        &self,
        topic: &str,
        key: Option<&str>,
        payload: &T,
    ) -> Result<(), EventError>
    where
        Self: Sized,
    {
        let serialized = serde_json::to_vec(payload)
            .map_err(|e| EventError::SerializationFailed(e.to_string()))?;
        self.publish(topic, key, &serialized).await
    }
}
