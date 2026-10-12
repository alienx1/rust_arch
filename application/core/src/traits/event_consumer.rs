//! Consuming side of the event stream (Redpanda / Kafka): a handler per message, run by a loop the
//! infrastructure owns.

use crate::errors::EventError;
use async_trait::async_trait;
use std::sync::Arc;

/// Handles one message. The consumer commits the offset after `handle` returns, whatever the
/// outcome (at-least-once, no poison message): a handler records its own failures.
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, key: Option<String>, payload: Vec<u8>);
}

#[async_trait]
pub trait EventConsumer: Send + Sync {
    /// Reads `topic` as consumer group `group` and hands every message to `handler`, one at a
    /// time and in order, until the process stops. Returns only on a fatal connection error.
    async fn consume(
        &self,
        topic: &str,
        group: &str,
        handler: Arc<dyn EventHandler>,
    ) -> Result<(), EventError>;
}
