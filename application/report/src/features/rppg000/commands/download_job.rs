//! RPPG000 download action: a short-lived signed link to the file

use super::rppg000_executor::Rppg000CommandExecutor;
use super::owned_job::storage_error;
use crate::features::rppg000::errors::Rppg000Error;
use crate::features::rppg000::models::DOWNLOAD_LINK_SECS;

impl Rppg000CommandExecutor {
    pub(super) async fn download_url_action(&self, requested_by: &str, id: &str) -> Result<String, Rppg000Error> {
        let (job, key) = self.owned_finished_job(requested_by, id).await?;
        // A link to a missing file only fails later in the browser: check it exists now
        self.content.size(&self.bucket, &key).await.map_err(storage_error)?;
        let name = job.file_name.unwrap_or_else(|| format!("{}.csv", job.id));
        self.content
            .presign_download_url(&self.bucket, &key, &name, DOWNLOAD_LINK_SECS)
            .await
            .map_err(storage_error)
    }
}
