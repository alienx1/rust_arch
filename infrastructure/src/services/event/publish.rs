use application_core::errors::EventError;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

pub async fn publish_event(
    producer: &FutureProducer,
    timeout: Duration,
    topic: &str,
    key: Option<&str>,
    payload: &[u8],
) -> Result<(), EventError> {
    let mut record = FutureRecord::to(topic).payload(payload);
    if let Some(k) = key {
        record = record.key(k);
    }

    producer
        .send(record, timeout)
        .await
        .map_err(|(err, _)| EventError::PublishFailed {
            topic: topic.to_string(),
            reason: err.to_string(),
        })?;

    Ok(())
}
