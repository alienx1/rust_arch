//! RPPG000 models: report job rows and the preview of a file

mod report_job_dto;
mod report_preview_dto;
mod rppg000_constants;

pub use domain::entity::rp_report_job::Model as ReportJobModel;
pub use report_job_dto::ReportJobDto;
pub use report_preview_dto::ReportPreviewDto;
pub use rppg000_constants::*;

/// `status` values of a report job
pub mod job_status {
    /// Waiting for the worker (set when an operator reprocesses a job, ALPG001)
    pub const QUEUED: &str = "queued";
    pub const RUNNING: &str = "running";
    pub const SUCCEEDED: &str = "succeeded";
    pub const FAILED: &str = "failed";
}
