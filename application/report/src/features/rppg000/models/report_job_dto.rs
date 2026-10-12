//! A report job as the list shows it

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ReportJobModel;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportJobDto {
    pub id: String,
    pub report_kind: String,
    /// `running` | `succeeded` | `failed`
    pub status: String,
    pub row_count: i64,
    pub size_bytes: i64,
    pub file_name: Option<String>,
    pub error: Option<String>,
    /// The filter the report was requested with
    pub filter: Value,
    /// RFC 3339, UTC
    pub requested_at: String,
    pub finished_at: Option<String>,
}

impl From<ReportJobModel> for ReportJobDto {
    fn from(m: ReportJobModel) -> Self {
        let iso = |t: chrono::DateTime<chrono::FixedOffset>| {
            t.with_timezone(&chrono::Utc).to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        };
        Self {
            id: m.id.to_string(),
            report_kind: m.report_kind,
            status: m.status,
            row_count: m.row_count,
            size_bytes: m.size_bytes,
            file_name: m.file_name,
            error: m.error,
            filter: m.filter,
            requested_at: iso(m.requested_at),
            finished_at: m.finished_at.map(iso),
        }
    }
}
