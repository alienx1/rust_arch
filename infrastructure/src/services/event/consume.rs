//! Redpanda consumer loop: one message at a time, offset committed after the handler returns.

use application_core::errors::EventError;
use application_core::traits::{EventConsumer, EventHandler};
use async_trait::async_trait;
use rdkafka::Message;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct RedpandaEventConsumer {
    brokers: String,
}

impl RedpandaEventConsumer {
    pub fn new(brokers: &str) -> Self {
        Self { brokers: brokers.to_string() }
    }
}

#[async_trait]
impl EventConsumer for RedpandaEventConsumer {
    async fn consume(
        &self,
        topic: &str,
        group: &str,
        handler: Arc<dyn EventHandler>,
    ) -> Result<(), EventError> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &self.brokers)
            .set("group.id", group)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            // A report can run for minutes: do not let the group think this worker died
            .set("max.poll.interval.ms", "3600000")
            .set("session.timeout.ms", "30000")
            .create()
            .map_err(|e| EventError::ConnectionError(format!("Failed to create Redpanda consumer: {e}")))?;
        consumer
            .subscribe(&[topic])
            .map_err(|e| EventError::ConnectionError(format!("Failed to subscribe to '{topic}': {e}")))?;
        tracing::info!(topic, group, "event consumer started");

        loop {
            match consumer.recv().await {
                Ok(msg) => {
                    let key = msg.key().map(|k| String::from_utf8_lossy(k).into_owned());
                    let payload = msg.payload().map(<[u8]>::to_vec).unwrap_or_default();
                    handler.handle(key, payload).await;
                    if let Err(e) = consumer.commit_message(&msg, CommitMode::Async) {
                        tracing::warn!(error = %e, "could not commit the offset");
                    }
                }
                Err(e) => {
                    // Broker hiccups surface here and librdkafka reconnects by itself
                    tracing::warn!(error = %e, "event consumer error");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }
}
