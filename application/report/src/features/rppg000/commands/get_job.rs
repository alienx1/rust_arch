//! RPPG000 get-one-job action

use super::rppg000_executor::Rppg000CommandExecutor;
use crate::features::rppg000::errors::Rppg000Error;
use crate::features::rppg000::models::ReportJobDto;

impl Rppg000CommandExecutor {
    pub(super) async fn get_job_action(&self, requested_by: &str, id: &str) -> Result<ReportJobDto, Rppg000Error> {
        Ok(self.owned_job(requested_by, id).await?.into())
    }
}
