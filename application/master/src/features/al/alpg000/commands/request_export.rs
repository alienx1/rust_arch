//! ALPG000 export request action: hands the filter to the report worker through Redpanda

use application_core::traits::{REPORT_REQUESTS_TOPIC, ReportJobMessage, report_kind};
use uuid::Uuid;

use super::alpg000_executor::Alpg000CommandExecutor;
use super::search_rules::normalize_search;
use crate::features::al::alpg000::errors::Alpg000Error;
use crate::features::al::alpg000::models::{AuditLogFilterQuery, ExportRequestedDto};

impl Alpg000CommandExecutor {
    pub(super) async fn request_export_action(
        &self,
        requested_by: &str,
        filter: AuditLogFilterQuery,
    ) -> Result<ExportRequestedDto, Alpg000Error> {
        let requested_by = requested_by.trim();
        if requested_by.is_empty() {
            return Err(Alpg000Error::Validation("The session has no username".to_string()));
        }
        // A bad filter is the requester's mistake: say so now, not in a failed job later
        let filter = normalize_search(filter)?;
        if filter.kind.as_deref().is_some_and(|k| k.starts_with("event_")) {
            return Err(Alpg000Error::Validation("Redpanda events cannot be exported".to_string()));
        }
        let events = self
            .events
            .as_ref()
            .ok_or_else(|| Alpg000Error::Unavailable("report queue is not configured".to_string()))?;

        let job_id = Uuid::now_v7().to_string();
        let message = ReportJobMessage {
            job_id: job_id.clone(),
            report_kind: report_kind::AUDIT_LOG_CSV.to_string(),
            requested_by: requested_by.to_string(),
            requested_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            filter,
            reprocess: false,
        };
        let payload = serde_json::to_vec(&message).map_err(|e| Alpg000Error::Internal(e.to_string()))?;
        events
            .publish(REPORT_REQUESTS_TOPIC, Some(&job_id), &payload)
            .await
            .map_err(|e| Alpg000Error::Unavailable(e.to_string()))?;
        Ok(ExportRequestedDto { job_id })
    }
}
