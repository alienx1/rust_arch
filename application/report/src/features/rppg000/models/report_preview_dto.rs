//! First rows of a finished report file

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ReportPreviewDto {
    pub file_name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    /// Rows in the whole file
    pub total_rows: i64,
    pub size_bytes: i64,
    /// `true` when the file has more rows than `rows`
    pub truncated: bool,
}
