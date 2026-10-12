//! RPPG000 DTOs: report jobs and the file preview

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const STATUS_RUNNING: &str = "running";
pub const STATUS_SUCCEEDED: &str = "succeeded";
pub const STATUS_FAILED: &str = "failed";


#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ReportJobDto {
    pub id: String,
    pub report_kind: String,
    pub status: String,
    pub row_count: i64,
    pub size_bytes: i64,
    pub file_name: Option<String>,
    pub error: Option<String>,
    pub filter: Value,
    /// RFC 3339, UTC
    pub requested_at: String,
    pub finished_at: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ReportPreviewDto {
    pub file_name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total_rows: i64,
    pub size_bytes: i64,
    pub truncated: bool,
}

/// `1.6 MB` style size text
pub fn size_text(bytes: i64) -> String {
    let b = bytes.max(0) as f64;
    if b < 1024.0 {
        format!("{bytes} B")
    } else if b < 1024.0 * 1024.0 {
        format!("{:.1} KB", b / 1024.0)
    } else if b < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} MB", b / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", b / (1024.0 * 1024.0 * 1024.0))
    }
}
