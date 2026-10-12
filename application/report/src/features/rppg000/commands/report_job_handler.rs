//! Adapter between the event consumer and the job runner: parses a `report.requests` message and
//! runs it.

use application_core::traits::{EventHandler, ReportJobMessage};
use async_trait::async_trait;
use std::sync::Arc;

use super::Rppg000Commands;

pub struct ReportJobHandler {
    commands: Arc<dyn Rppg000Commands>,
}

impl ReportJobHandler {
    pub fn new(commands: Arc<dyn Rppg000Commands>) -> Self {
        Self { commands }
    }
}

#[async_trait]
impl EventHandler for ReportJobHandler {
    async fn handle(&self, _key: Option<String>, payload: Vec<u8>) {
        match serde_json::from_slice::<ReportJobMessage>(&payload) {
            Ok(message) => self.commands.run_job(message).await,
            // Nothing sensible to retry: a message we cannot read stays unreadable
            Err(e) => tracing::error!(error = %e, "unreadable report request skipped"),
        }
    }
}
