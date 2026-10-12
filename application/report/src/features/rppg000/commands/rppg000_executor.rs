//! RPPG000 Command Executor: wires the ports (job rows, audit store, object storage,
//! notifications) to the actions, each of which lives in its own file.

use application_core::traits::{AuditLogPort, ContentService, NotificationService, ReportJobMessage};
use async_trait::async_trait;
use std::sync::Arc;

use super::Rppg000Commands;
use crate::features::rppg000::errors::Rppg000Error;
use crate::features::rppg000::models::{ReportJobDto, ReportPreviewDto};
use crate::features::rppg000::queries::Rppg000Queries;

pub struct Rppg000CommandExecutor {
    pub(super) jobs: Arc<dyn Rppg000Queries>,
    pub(super) audit: Arc<dyn AuditLogPort>,
    pub(super) content: Arc<dyn ContentService>,
    pub(super) notifier: Arc<dyn NotificationService>,
    /// Bucket the report files go to
    pub(super) bucket: String,
}

impl Rppg000CommandExecutor {
    pub fn new(
        jobs: Arc<dyn Rppg000Queries>,
        audit: Arc<dyn AuditLogPort>,
        content: Arc<dyn ContentService>,
        notifier: Arc<dyn NotificationService>,
        bucket: impl Into<String>,
    ) -> Self {
        Self { jobs, audit, content, notifier, bucket: bucket.into() }
    }
}

#[async_trait]
impl Rppg000Commands for Rppg000CommandExecutor {
    async fn run_job(&self, message: ReportJobMessage) {
        self.run_job_action(message).await
    }

    async fn list_jobs(&self, requested_by: &str) -> Result<Vec<ReportJobDto>, Rppg000Error> {
        self.list_jobs_action(requested_by).await
    }

    async fn get_job(&self, requested_by: &str, id: &str) -> Result<ReportJobDto, Rppg000Error> {
        self.get_job_action(requested_by, id).await
    }

    async fn preview_job(
        &self,
        requested_by: &str,
        id: &str,
        rows: Option<usize>,
    ) -> Result<ReportPreviewDto, Rppg000Error> {
        self.preview_job_action(requested_by, id, rows).await
    }

    async fn download_url(&self, requested_by: &str, id: &str) -> Result<String, Rppg000Error> {
        self.download_url_action(requested_by, id).await
    }
}
