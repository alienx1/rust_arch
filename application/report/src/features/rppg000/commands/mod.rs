//! RPPG000 Commands trait (the worker's job runner + what the requester reads) and executor

use async_trait::async_trait;
use application_core::traits::ReportJobMessage;

use crate::features::rppg000::errors::Rppg000Error;
use crate::features::rppg000::models::{ReportJobDto, ReportPreviewDto};

pub mod audit_csv_rows;
mod csv_object_writer;
mod csv_preview;
mod download_job;
mod get_job;
mod list_jobs;
mod owned_job;
mod preview_job;
mod report_job_handler;
pub mod rppg000_executor;
mod run_report_job;

pub use csv_preview::preview_rows;
pub use report_job_handler::ReportJobHandler;
pub use rppg000_executor::Rppg000CommandExecutor;

#[async_trait]
pub trait Rppg000Commands: Send + Sync {
    /// Builds the report of `message`: streams the audit CSV into object storage, records the
    /// outcome and notifies the requester. Never fails the caller: a failure is the job's
    /// `failed` status (and a notification), so the message is not redelivered forever.
    async fn run_job(&self, message: ReportJobMessage);

    /// The requester's jobs, newest first
    async fn list_jobs(&self, requested_by: &str) -> Result<Vec<ReportJobDto>, Rppg000Error>;

    /// One of the requester's jobs; another user's job is `NotFound`
    async fn get_job(&self, requested_by: &str, id: &str) -> Result<ReportJobDto, Rppg000Error>;

    /// First rows of a finished job's file (`rows` default 100, at most 500)
    async fn preview_job(
        &self,
        requested_by: &str,
        id: &str,
        rows: Option<usize>,
    ) -> Result<ReportPreviewDto, Rppg000Error>;

    /// Short-lived link that downloads a finished job's file
    async fn download_url(&self, requested_by: &str, id: &str) -> Result<String, Rppg000Error>;
}
