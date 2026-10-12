//! ALPG000 Commands trait (reads + the client-event intake consumed by routers) and executor

use async_trait::async_trait;

use crate::features::al::alpg000::errors::Alpg000Error;
use crate::features::al::alpg000::models::{
    AuditLogDto, AuditLogFilterQuery, AuditLogPageDto, AuditStatusDto, ClientActor,
    ClientEventCommand, ExportRequestedDto, ProgramOptionDto,
};

pub mod alpg000_executor;
mod audit_store_status;
mod client_event_rules;
mod request_export;
mod get_log;
mod list_program_options;
mod record_client_events;
mod search_logs;
mod search_rules;

pub use alpg000_executor::Alpg000CommandExecutor;
pub use client_event_rules::normalize_client_events;
pub use search_rules::normalize_search;

#[async_trait]
pub trait Alpg000Commands: Send + Sync {
    /// Search the audit events (newest first). Store down -> `Unavailable`.
    async fn search_logs(&self, filter: AuditLogFilterQuery) -> Result<AuditLogPageDto, Alpg000Error>;

    /// Queues a CSV export of everything `filter` matches (paging fields ignored) for the user
    /// `requested_by`: publishes the request to Redpanda and returns the job id at once. The
    /// report worker builds the file and notifies the user. Broker down -> `Unavailable`.
    async fn request_export(
        &self,
        requested_by: &str,
        filter: AuditLogFilterQuery,
    ) -> Result<ExportRequestedDto, Alpg000Error>;

    /// One event with headers and bodies by id
    async fn get_log(&self, id: &str) -> Result<Option<AuditLogDto>, Alpg000Error>;

    /// Catalogue programs for the program filter (empty when no catalogue source is attached)
    async fn program_options(&self) -> Result<Vec<ProgramOptionDto>, Alpg000Error>;

    /// Whether the audit store answers
    async fn status(&self) -> AuditStatusDto;

    /// Stores the page / click / action events the web app reports for `actor`. Returns how many
    /// were accepted (events of an unknown kind are dropped); more than 50 events -> `Validation`.
    /// Trouble with the store never surfaces here.
    async fn record_client_events(
        &self,
        actor: ClientActor,
        events: Vec<ClientEventCommand>,
    ) -> Result<usize, Alpg000Error>;
}
