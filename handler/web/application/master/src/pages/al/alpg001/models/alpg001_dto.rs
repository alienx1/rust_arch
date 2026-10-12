//! ALPG001 DTOs: process rows, the per-status counters and the list filters

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const STATUS_QUEUED: &str = "queued";
pub const STATUS_RUNNING: &str = "running";
pub const STATUS_SUCCEEDED: &str = "succeeded";
pub const STATUS_FAILED: &str = "failed";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProcessDto {
    pub id: String,
    pub kind: String,
    pub requested_by: String,
    pub status: String,
    pub row_count: i64,
    pub size_bytes: i64,
    pub file_name: Option<String>,
    pub error: Option<String>,
    /// RFC 3339, UTC
    pub requested_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub duration_ms: Option<i64>,
    /// Waiting / running for a long time without moving
    pub stuck: bool,
    pub can_reprocess: bool,
    pub filter: Value,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessCountsDto {
    pub queued: u64,
    pub running: u64,
    pub succeeded: u64,
    pub failed: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProcessPageDto {
    pub items: Vec<ProcessDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
    pub counts: ProcessCountsDto,
}

/// Filter state of the list (`ALL` / empty = no filter). Dates are `YYYY-MM-DD` in the browser's
/// time zone; both empty = every day.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcessFilters {
    pub status: String,
    pub date_from: String,
    pub date_to: String,
    /// Requester's username
    pub user: String,
}

impl ProcessFilters {
    pub fn all() -> Self {
        Self { status: "ALL".to_string(), ..Self::default() }
    }
}

/// Which filter a change applies to
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessFilterField {
    Status,
    DateFrom,
    DateTo,
    User,
}

impl ProcessFilters {
    pub fn set(&mut self, field: ProcessFilterField, value: String) {
        match field {
            ProcessFilterField::Status => self.status = value,
            ProcessFilterField::DateFrom => self.date_from = value,
            ProcessFilterField::DateTo => self.date_to = value,
            ProcessFilterField::User => self.user = value,
        }
    }
}

impl ProcessPageDto {
    /// A process is still waiting or running: the list should keep refreshing
    pub fn has_active(&self) -> bool {
        self.items.iter().any(|p| (p.status == STATUS_QUEUED || p.status == STATUS_RUNNING) && !p.stuck)
    }
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

/// `850 ms` / `12.4 s` / `3 m 05 s`
pub fn duration_text(ms: i64) -> String {
    let ms = ms.max(0);
    if ms < 1000 {
        format!("{ms} ms")
    } else if ms < 60_000 {
        format!("{:.1} s", ms as f64 / 1000.0)
    } else {
        format!("{} m {:02} s", ms / 60_000, (ms % 60_000) / 1000)
    }
}
