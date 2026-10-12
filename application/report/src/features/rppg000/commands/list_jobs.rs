//! RPPG000 list action

use super::rppg000_executor::Rppg000CommandExecutor;
use crate::features::rppg000::errors::Rppg000Error;
use crate::features::rppg000::models::{LIST_LIMIT, ReportJobDto};

impl Rppg000CommandExecutor {
    pub(super) async fn list_jobs_action(&self, requested_by: &str) -> Result<Vec<ReportJobDto>, Rppg000Error> {
        Ok(self.jobs.list_jobs(requested_by, LIST_LIMIT).await?.into_iter().map(Into::into).collect())
    }
}
