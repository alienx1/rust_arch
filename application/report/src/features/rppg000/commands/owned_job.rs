//! Finds a job that belongs to the requester

use application_core::errors::ContentError;
use uuid::Uuid;

use super::rppg000_executor::Rppg000CommandExecutor;
use crate::features::rppg000::errors::Rppg000Error;
use crate::features::rppg000::models::{ReportJobModel, job_status};

impl Rppg000CommandExecutor {
    /// The job `id` of `requested_by`. Someone else's job looks exactly like a missing one.
    pub(super) async fn owned_job(&self, requested_by: &str, id: &str) -> Result<ReportJobModel, Rppg000Error> {
        let not_found = || Rppg000Error::NotFound(format!("Report {id} not found"));
        let uuid = Uuid::parse_str(id.trim()).map_err(|_| not_found())?;
        self.jobs
            .find_job(uuid)
            .await?
            .filter(|job| job.requested_by == requested_by)
            .ok_or_else(not_found)
    }

    /// Same, but the file must exist: returns the job with its object key
    pub(super) async fn owned_finished_job(
        &self,
        requested_by: &str,
        id: &str,
    ) -> Result<(ReportJobModel, String), Rppg000Error> {
        let job = self.owned_job(requested_by, id).await?;
        match (job.status.as_str(), job.object_key.clone()) {
            (job_status::SUCCEEDED, Some(key)) => Ok((job, key)),
            (job_status::FAILED, _) => Err(Rppg000Error::NotReady("The report failed, there is no file".into())),
            _ => Err(Rppg000Error::NotReady("The report is still being built".into())),
        }
    }
}

/// Storage failures as feature errors: a missing object is the file being gone, anything else is
/// the storage being down
pub(super) fn storage_error(e: ContentError) -> Rppg000Error {
    match e {
        ContentError::NotFound(m) => Rppg000Error::FileGone(m),
        other => Rppg000Error::Storage(other.to_string()),
    }
}
