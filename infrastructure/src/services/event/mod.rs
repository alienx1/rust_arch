//! Redpanda (Kafka wire protocol) Event Service implementation.

pub mod consume;
pub mod publish;

pub use consume::RedpandaEventConsumer;

use application_core::errors::EventError;
use application_core::traits::EventService;
use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::producer::FutureProducer;
use std::time::Duration;

#[derive(Clone)]
pub struct RedpandaEventService {
    producer: FutureProducer,
    timeout: Duration,
}

impl RedpandaEventService {
    /// Create a new Redpanda event service connected to the given brokers (e.g. `redpanda:9092`).
    pub fn new(brokers: &str) -> Result<Self, EventError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("acks", "all")
            .create()
            .map_err(|e| {
                EventError::ConnectionError(format!("Failed to create Redpanda producer: {e}"))
            })?;

        Ok(Self {
            producer,
            timeout: Duration::from_secs(5),
        })
    }

    /// Construct service from an existing `FutureProducer`.
    pub fn from_producer(producer: FutureProducer) -> Self {
        Self {
            producer,
            timeout: Duration::from_secs(5),
        }
    }

    /// Access underlying `FutureProducer`.
    pub fn producer(&self) -> &FutureProducer {
        &self.producer
    }
}

#[async_trait]
impl EventService for RedpandaEventService {
    async fn publish(
        &self,
        topic: &str,
        key: Option<&str>,
        payload: &[u8],
    ) -> Result<(), EventError> {
        publish::publish_event(&self.producer, self.timeout, topic, key, payload).await
    }
}
