//! ALPG001 Commands trait (reads + the reprocess action consumed by routers) and executor

use async_trait::async_trait;

use crate::features::al::alpg001::errors::Alpg001Error;
use crate::features::al::alpg001::models::{
    ProcessDto, ProcessFilterQuery, ProcessPageDto, ReprocessRequestedDto,
};

pub mod alpg001_executor;
mod filter_rules;
mod get_process;
mod list_processes;
mod reprocess_process;

pub use alpg001_executor::Alpg001CommandExecutor;
pub use filter_rules::normalize_process_filter;

#[async_trait]
pub trait Alpg001Commands: Send + Sync {
    /// Processes under the filter (newest activity first) with the per-status counters
    async fn list_processes(&self, filter: ProcessFilterQuery) -> Result<ProcessPageDto, Alpg001Error>;

    async fn get_process(&self, id: &str) -> Result<Option<ProcessDto>, Alpg001Error>;

    /// Runs process `id` again from its stored request: the row goes back to `queued` and the
    /// request is published to Redpanda once more. Running (not stuck) -> `Conflict`; broker down
    /// -> `Unavailable` (the row is left as it was).
    async fn reprocess(&self, id: &str) -> Result<ReprocessRequestedDto, Alpg001Error>;
}
