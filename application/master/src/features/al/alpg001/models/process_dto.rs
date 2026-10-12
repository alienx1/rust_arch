//! ALPG001 one process row

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{ProcessModel, STUCK_AFTER_SECS, process_status};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProcessDto {
    pub id: String,
    /// What kind of process, e.g. `audit_log_csv`
    pub kind: String,
    pub requested_by: String,
    /// `queued` / `running` / `succeeded` / `failed`
    pub status: String,
    pub row_count: i64,
    pub size_bytes: i64,
    pub file_name: Option<String>,
    /// Why it failed
    pub error: Option<String>,
    pub requested_at: String,
    /// Last time the process was queued or started
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    /// Run time of a finished process
    pub duration_ms: Option<i64>,
    /// `queued` / `running` for longer than [`STUCK_AFTER_SECS`]
    pub stuck: bool,
    /// Whether the "reprocess" action applies now
    pub can_reprocess: bool,
    /// The request the process was started with (what a reprocess runs again)
    pub filter: Value,
}

fn rfc3339(t: DateTime<Utc>) -> String {
    t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// A waiting / running process that has not moved for [`STUCK_AFTER_SECS`]
pub fn is_stuck(status: &str, started_at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    (status == process_status::QUEUED || status == process_status::RUNNING)
        && started_at.is_some_and(|s| (now - s).num_seconds() >= STUCK_AFTER_SECS)
}

/// Finished processes can be run again; waiting / running ones only when stuck
pub fn can_reprocess(status: &str, stuck: bool) -> bool {
    match status {
        process_status::SUCCEEDED | process_status::FAILED => true,
        process_status::QUEUED | process_status::RUNNING => stuck,
        _ => false,
    }
}

impl ProcessDto {
    pub fn from_model(m: ProcessModel, now: DateTime<Utc>) -> Self {
        let started = m.started_at.map(|t| t.with_timezone(&Utc));
        let finished = m.finished_at.map(|t| t.with_timezone(&Utc));
        let stuck = is_stuck(&m.status, started, now);
        let finished_ok = m.status == process_status::SUCCEEDED || m.status == process_status::FAILED;
        Self {
            id: m.id.to_string(),
            kind: m.report_kind,
            requested_by: m.requested_by,
            can_reprocess: can_reprocess(&m.status, stuck),
            status: m.status,
            row_count: m.row_count,
            size_bytes: m.size_bytes,
            file_name: m.file_name,
            error: m.error,
            requested_at: rfc3339(m.requested_at.with_timezone(&Utc)),
            started_at: started.map(rfc3339),
            finished_at: finished.map(rfc3339),
            duration_ms: match (started, finished) {
                (Some(s), Some(f)) if finished_ok => Some((f - s).num_milliseconds().max(0)),
                _ => None,
            },
            stuck,
            filter: m.filter,
        }
    }
}
