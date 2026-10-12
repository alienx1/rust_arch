//! Adapter between the event consumer and the executor: parses a `notifications` message and
//! ingests it.

use application_core::traits::{EventHandler, NotificationMessage};
use async_trait::async_trait;
use std::sync::Arc;

use super::Ntpg000Commands;

pub struct NotificationMessageHandler {
    commands: Arc<dyn Ntpg000Commands>,
}

impl NotificationMessageHandler {
    pub fn new(commands: Arc<dyn Ntpg000Commands>) -> Self {
        Self { commands }
    }
}

#[async_trait]
impl EventHandler for NotificationMessageHandler {
    async fn handle(&self, _key: Option<String>, payload: Vec<u8>) {
        match serde_json::from_slice::<NotificationMessage>(&payload) {
            Ok(message) => self.commands.ingest(message).await,
            // A message we cannot read stays unreadable: nothing to retry
            Err(e) => tracing::error!(error = %e, "unreadable notification skipped"),
        }
    }
}
