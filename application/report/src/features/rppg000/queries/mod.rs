//! RPPG000 Queries trait: report job rows

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::features::rppg000::errors::Rppg000Error;
use crate::features::rppg000::models::ReportJobModel;

pub mod db_rppg000_queries;

pub use db_rppg000_queries::DbRppg000Queries;

#[async_trait]
pub trait Rppg000Queries: Send + Sync {
    /// Records that the worker started job `id` (a redelivered request starts it again)
    async fn start_job(
        &self,
        id: Uuid,
        report_kind: &str,
        requested_by: &str,
        filter: Value,
        requested_at: DateTime<Utc>,
    ) -> Result<(), Rppg000Error>;

    async fn find_job(&self, id: Uuid) -> Result<Option<ReportJobModel>, Rppg000Error>;

    async fn finish_job_ok(
        &self,
        id: Uuid,
        row_count: i64,
        size_bytes: i64,
        file_name: &str,
        object_key: &str,
    ) -> Result<(), Rppg000Error>;

    async fn finish_job_failed(&self, id: Uuid, error: &str) -> Result<(), Rppg000Error>;

    /// Newest first
    async fn list_jobs(&self, requested_by: &str, limit: u64) -> Result<Vec<ReportJobModel>, Rppg000Error>;
}
