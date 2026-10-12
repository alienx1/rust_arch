//! ALPG001 executor: wires the queries and the event stream behind `Alpg001Commands`

use application_core::traits::EventService;
use async_trait::async_trait;
use std::sync::Arc;

use super::Alpg001Commands;
use crate::features::al::alpg001::errors::Alpg001Error;
use crate::features::al::alpg001::models::{
    ProcessDto, ProcessFilterQuery, ProcessPageDto, ReprocessRequestedDto,
};
use crate::features::al::alpg001::queries::Alpg001Queries;

pub struct Alpg001CommandExecutor {
    pub(super) queries: Arc<dyn Alpg001Queries>,
    /// Where reprocess requests go (Redpanda); without it a reprocess is `Unavailable`
    pub(super) events: Option<Arc<dyn EventService>>,
}

impl Alpg001CommandExecutor {
    pub fn new(queries: Arc<dyn Alpg001Queries>) -> Self {
        Self { queries, events: None }
    }

    /// Attaches the event stream reprocess requests are published to
    pub fn with_events(mut self, events: Arc<dyn EventService>) -> Self {
        self.events = Some(events);
        self
    }
}

#[async_trait]
impl Alpg001Commands for Alpg001CommandExecutor {
    async fn list_processes(&self, filter: ProcessFilterQuery) -> Result<ProcessPageDto, Alpg001Error> {
        self.list_processes_action(filter).await
    }

    async fn get_process(&self, id: &str) -> Result<Option<ProcessDto>, Alpg001Error> {
        self.get_process_action(id).await
    }

    async fn reprocess(&self, id: &str) -> Result<ReprocessRequestedDto, Alpg001Error> {
        self.reprocess_action(id).await
    }
}
