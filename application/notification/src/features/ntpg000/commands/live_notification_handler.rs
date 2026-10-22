//! Adapter between the event consumer and the executor: parses a `notification-live` message and
//! pushes it, storing nothing.

use application_core::traits::{EventHandler, NotificationMessage};
use async_trait::async_trait;
use std::sync::Arc;

use super::Ntpg000Commands;

pub struct LiveNotificationHandler {
    commands: Arc<dyn Ntpg000Commands>,
}

impl LiveNotificationHandler {
    pub fn new(commands: Arc<dyn Ntpg000Commands>) -> Self {
        Self { commands }
    }
}

#[async_trait]
impl EventHandler for LiveNotificationHandler {
    async fn handle(&self, _key: Option<String>, payload: Vec<u8>) {
        match serde_json::from_slice::<NotificationMessage>(&payload) {
            Ok(message) => self.commands.push_live(message).await,
            // A message we cannot read stays unreadable: nothing to retry
            Err(e) => tracing::error!(error = %e, "unreadable live notice skipped"),
        }
    }
}
