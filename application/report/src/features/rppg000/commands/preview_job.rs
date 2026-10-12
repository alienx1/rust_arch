//! RPPG000 preview action: the first rows of the file, read with a ranged GET

use super::csv_preview::preview_rows;
use super::rppg000_executor::Rppg000CommandExecutor;
use super::owned_job::storage_error;
use crate::features::rppg000::errors::Rppg000Error;
use crate::features::rppg000::models::{
    DEFAULT_PREVIEW_ROWS, MAX_PREVIEW_ROWS, PREVIEW_READ_BYTES, ReportPreviewDto,
};

impl Rppg000CommandExecutor {
    pub(super) async fn preview_job_action(
        &self,
        requested_by: &str,
        id: &str,
        rows: Option<usize>,
    ) -> Result<ReportPreviewDto, Rppg000Error> {
        let (job, key) = self.owned_finished_job(requested_by, id).await?;
        let wanted = rows.filter(|r| *r > 0).unwrap_or(DEFAULT_PREVIEW_ROWS).min(MAX_PREVIEW_ROWS);
        let bytes = self
            .content
            .get_range(&self.bucket, &key, 0, PREVIEW_READ_BYTES - 1)
            .await
            .map_err(storage_error)?;
        let cut = job.size_bytes as u64 > bytes.len() as u64;
        let (columns, rows) = preview_rows(&bytes, wanted, cut);
        Ok(ReportPreviewDto {
            file_name: job.file_name.unwrap_or_default(),
            truncated: (rows.len() as i64) < job.row_count,
            columns,
            rows,
            total_rows: job.row_count,
            size_bytes: job.size_bytes,
        })
    }
}
