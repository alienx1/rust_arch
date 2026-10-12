//! Contract between the API that accepts a report request and the report worker that builds it
//! (the message on [`REPORT_REQUESTS_TOPIC`]).

use super::audit_log::AuditSearch;
use serde::{Deserialize, Serialize};

/// Redpanda topic the report worker consumes
pub const REPORT_REQUESTS_TOPIC: &str = "report.requests";
/// Consumer group of the report worker
pub const REPORT_WORKER_GROUP: &str = "report-worker";

/// `report_kind` values
pub mod report_kind {
    /// CSV of the audit events matching a filter (ALPG000)
    pub const AUDIT_LOG_CSV: &str = "audit_log_csv";
}

/// One report request
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportJobMessage {
    /// UUIDv7, also the Redpanda message key
    pub job_id: String,
    pub report_kind: String,
    /// Kratos username of the requester: the only person who can see the result
    pub requested_by: String,
    /// RFC 3339, UTC
    pub requested_at: String,
    /// What to export (`page` / `limit` are ignored)
    pub filter: AuditSearch,
    /// `true` when an operator re-runs a job that already has a row (ALPG001): the worker then
    /// runs it only while the row is `queued` / `running`, so a redelivery after it finished is a
    /// no-op. Absent in messages of a first request.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reprocess: bool,
}
