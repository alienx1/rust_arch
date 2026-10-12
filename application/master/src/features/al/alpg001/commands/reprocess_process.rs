//! ALPG001 reprocess action

use application_core::traits::{AuditSearch, REPORT_REQUESTS_TOPIC, ReportJobMessage};
use chrono::{Duration, Utc};
use uuid::Uuid;

use super::alpg001_executor::Alpg001CommandExecutor;
use crate::features::al::alpg001::errors::Alpg001Error;
use crate::features::al::alpg001::models::{ReprocessRequestedDto, STUCK_AFTER_SECS};

impl Alpg001CommandExecutor {
    pub(super) async fn reprocess_action(&self, id: &str) -> Result<ReprocessRequestedDto, Alpg001Error> {
        let id = Uuid::parse_str(id.trim()).map_err(|_| Alpg001Error::Validation("Process id must be a UUID".into()))?;
        let events = self
            .events
            .as_ref()
            .ok_or_else(|| Alpg001Error::Unavailable("report queue is not configured".to_string()))?;
        let before = self
            .queries
            .find(id)
            .await?
            .ok_or_else(|| Alpg001Error::NotFound(format!("Process {id} not found")))?;
        // The request the process was started with; a row whose request cannot be read cannot be
        // run again
        let filter: AuditSearch = serde_json::from_value(before.filter.clone())
            .map_err(|e| Alpg001Error::Validation(format!("The stored request cannot be read: {e}")))?;

        let now = Utc::now();
        let queued = self.queries.mark_queued(id, now, now - Duration::seconds(STUCK_AFTER_SECS)).await?;
        if !queued {
            return Err(Alpg001Error::Conflict(format!(
                "Process {id} is {} and cannot be reprocessed now",
                before.status
            )));
        }

        let message = ReportJobMessage {
            job_id: id.to_string(),
            report_kind: before.report_kind.clone(),
            requested_by: before.requested_by.clone(),
            requested_at: before.requested_at.with_timezone(&Utc).to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            filter,
            reprocess: true,
        };
        let payload = serde_json::to_vec(&message).map_err(|e| Alpg001Error::Internal(e.to_string()))?;
        if let Err(e) = events.publish(REPORT_REQUESTS_TOPIC, Some(&message.job_id), &payload).await {
            // Nobody will pick it up: leave the row as it was instead of `queued` forever
            if let Err(restore) = self.queries.restore(before).await {
                tracing::error!(%id, error = %restore, "cannot restore the process after a failed reprocess request");
            }
            return Err(Alpg001Error::Unavailable(e.to_string()));
        }
        Ok(ReprocessRequestedDto { id: id.to_string() })
    }
}
