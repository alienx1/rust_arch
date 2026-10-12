//! ALPG000 Command Executor: validates the search filter and the client events and hands them to
//! the audit store port. Each action lives in its own file; this file wires them to the trait.

use application_core::traits::{AuditLogPort, EventService};
use async_trait::async_trait;
use std::sync::Arc;

use crate::features::al::alpg000::commands::Alpg000Commands;
use crate::features::al::alpg000::errors::Alpg000Error;
use crate::features::al::alpg000::models::{
    AuditLogDto, AuditLogFilterQuery, AuditLogPageDto, AuditStatusDto, ClientActor,
    ClientEventCommand, ExportRequestedDto, ProgramOptionDto,
};
use crate::features::al::alpg000::queries::Alpg000Queries;

pub struct Alpg000CommandExecutor {
    pub(super) audit: Arc<dyn AuditLogPort>,
    pub(super) programs: Option<Arc<dyn Alpg000Queries>>,
    /// Where export requests go (Redpanda); without it exports are `Unavailable`
    pub(super) events: Option<Arc<dyn EventService>>,
    /// Valkey read-through for the program catalogue (off until `with_cache`)
    pub(super) caches: application_core::traits::ReadCaches,
}

impl Alpg000CommandExecutor {
    pub fn new(audit: Arc<dyn AuditLogPort>) -> Self {
        Self { audit, programs: None, events: None, caches: application_core::traits::ReadCaches::disabled() }
    }

    /// Serves the program catalogue from Valkey; a write to the catalogue clears it
    pub fn with_cache(mut self, caches: application_core::traits::ReadCaches) -> Self {
        self.caches = caches;
        self
    }

    /// Attaches the event stream export requests are published to
    pub fn with_events(mut self, events: Arc<dyn EventService>) -> Self {
        self.events = Some(events);
        self
    }

    /// Attaches the catalogue source of the program filter
    pub fn with_programs(mut self, programs: Arc<dyn Alpg000Queries>) -> Self {
        self.programs = Some(programs);
        self
    }
}

#[async_trait]
impl Alpg000Commands for Alpg000CommandExecutor {
    async fn search_logs(&self, filter: AuditLogFilterQuery) -> Result<AuditLogPageDto, Alpg000Error> {
        self.search_logs_action(filter).await
    }

    async fn request_export(
        &self,
        requested_by: &str,
        filter: AuditLogFilterQuery,
    ) -> Result<ExportRequestedDto, Alpg000Error> {
        self.request_export_action(requested_by, filter).await
    }

    async fn get_log(&self, id: &str) -> Result<Option<AuditLogDto>, Alpg000Error> {
        self.get_log_action(id).await
    }

    async fn program_options(&self) -> Result<Vec<ProgramOptionDto>, Alpg000Error> {
        self.list_program_options().await
    }

    async fn status(&self) -> AuditStatusDto {
        self.audit_store_status().await
    }

    async fn record_client_events(
        &self,
        actor: ClientActor,
        events: Vec<ClientEventCommand>,
    ) -> Result<usize, Alpg000Error> {
        self.record_client_events_action(actor, events)
    }
}
