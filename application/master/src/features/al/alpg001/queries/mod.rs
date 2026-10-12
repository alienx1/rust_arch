//! ALPG001 Queries trait: process rows (`rp_report_job`)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::features::al::alpg001::errors::Alpg001Error;
use crate::features::al::alpg001::models::{ProcessCountsDto, ProcessModel, ProcessSearch};

pub mod db_alpg001_queries;

pub use db_alpg001_queries::DbAlpg001Queries;

#[async_trait]
pub trait Alpg001Queries: Send + Sync {
    /// One page (newest activity first), the total under the filter and the per-status counters
    /// under the filter without its status
    async fn search(
        &self,
        filter: &ProcessSearch,
    ) -> Result<(Vec<ProcessModel>, u64, ProcessCountsDto), Alpg001Error>;

    async fn find(&self, id: Uuid) -> Result<Option<ProcessModel>, Alpg001Error>;

    /// Puts the process back to `queued` (error cleared, `started_at = now`) when it may be
    /// reprocessed: finished, or waiting / running since before `stuck_before`. One conditional
    /// update, so two operators cannot queue the same process twice. `false` = not allowed now.
    async fn mark_queued(
        &self,
        id: Uuid,
        now: DateTime<Utc>,
        stuck_before: DateTime<Utc>,
    ) -> Result<bool, Alpg001Error>;

    /// Writes `row` back as it was (the request could not be queued after all)
    async fn restore(&self, row: ProcessModel) -> Result<(), Alpg001Error>;
}
