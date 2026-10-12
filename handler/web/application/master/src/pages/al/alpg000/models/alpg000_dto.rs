//! ALPG000 DTOs: mirror the audit API (`/api/v1/al/alpg000`). Every optional field may be
//! null or absent, so all of them default.

use serde::{Deserialize, Serialize};

/// Event kinds written by the server (`api`) and the web app (the rest)
/// Filter-only group: page views + clicks + actions
pub const KIND_UI: &str = "ui";
pub const KIND_PAGE_VIEW: &str = "page_view";
pub const KIND_CLICK: &str = "click";
pub const KIND_ACTION: &str = "action";
pub const KIND_API: &str = "api";
pub const KIND_DATA_CHANGE: &str = "data_change";
/// Redpanda events, kept in their own Quickwit indexes
pub const KIND_EVENT_REPORT_REQUEST: &str = "event_report_request";
pub const KIND_EVENT_NOTIFICATION: &str = "event_notification";

/// One row of the log list
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AuditLogSummaryDto {
    #[serde(default)]
    pub event_id: String,
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub program_code: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub status: Option<u16>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub page: Option<String>,
    #[serde(default)]
    pub element: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub table_name: Option<String>,
}

/// Server-paged list answer
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuditLogPageDto {
    #[serde(default)]
    pub items: Vec<AuditLogSummaryDto>,
    #[serde(default)]
    pub total: u64,
    #[serde(default = "first_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
}

fn first_page() -> u64 {
    1
}

fn default_limit() -> u64 {
    25
}

impl Default for AuditLogPageDto {
    fn default() -> Self {
        Self { items: Vec::new(), total: 0, page: first_page(), limit: default_limit() }
    }
}

/// Full event of the detail page
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AuditLogDto {
    #[serde(default)]
    pub event_id: String,
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub program_code: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub status: Option<u16>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
    /// JSON object of header name -> value (secrets already masked)
    #[serde(default)]
    pub request_headers: Option<serde_json::Value>,
    #[serde(default)]
    pub response_headers: Option<serde_json::Value>,
    /// JSON text (or an already parsed JSON value)
    #[serde(default)]
    pub request_body: Option<serde_json::Value>,
    #[serde(default)]
    pub response_body: Option<serde_json::Value>,
    #[serde(default)]
    pub page: Option<String>,
    #[serde(default)]
    pub element: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub detail: Option<String>,
    /// Data change: tables touched, the changed rows (JSON text of `DataChangeEntryDto`s) and the
    /// causing API call
    #[serde(default)]
    pub table_name: Option<String>,
    #[serde(default)]
    pub changes: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
}

/// `GET /programs`: a catalogue program of the program filter
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProgramOptionDto {
    pub code: String,
    #[serde(default)]
    pub name_th: String,
    #[serde(default)]
    pub name_en: String,
}

impl ProgramOptionDto {
    /// `DBPG004 — Reference Data` in the language of the system
    pub fn display_name(&self) -> String {
        let name = web_package::core::i18n::localized(&self.name_th, &self.name_en);
        if name.is_empty() { self.code.clone() } else { format!("{} — {}", self.code, name) }
    }
}

/// `GET /status`
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AuditStatusDto {
    #[serde(default)]
    pub available: bool,
}

/// Filter state of the list (all values are the raw choice; `ALL` / empty = no filter)
#[derive(Clone, Debug, PartialEq)]
pub struct AuditFilters {
    /// First day, `YYYY-MM-DD` in the browser's time zone (empty = no lower bound)
    pub date_from: String,
    /// Last day (inclusive), `YYYY-MM-DD` (empty = no upper bound)
    pub date_to: String,
    pub kind: String,
    /// `2`..`5` or `ALL`
    pub status_class: String,
    pub user: String,
    pub method: String,
    pub program_code: String,
    pub path: String,
    /// Full-text search
    pub text: String,
}

impl Default for AuditFilters {
    fn default() -> Self {
        Self {
            date_from: today(),
            date_to: today(),
            kind: "ALL".to_string(),
            status_class: "ALL".to_string(),
            user: String::new(),
            method: "ALL".to_string(),
            program_code: "ALL".to_string(),
            path: String::new(),
            text: String::new(),
        }
    }
}

/// Today as `YYYY-MM-DD` in the browser's time zone
pub fn today() -> String {
    let now = js_sys::Date::new_0();
    format!("{:04}-{:02}-{:02}", now.get_full_year(), now.get_month() + 1, now.get_date())
}

/// Which filter a change applies to
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuditFilterField {
    DateFrom,
    DateTo,
    Kind,
    StatusClass,
    User,
    Method,
    ProgramCode,
    Path,
    Text,
}

impl AuditFilters {
    pub fn set(&mut self, field: AuditFilterField, value: String) {
        match field {
            AuditFilterField::DateFrom => self.date_from = value,
            AuditFilterField::DateTo => self.date_to = value,
            AuditFilterField::Kind => self.kind = value,
            AuditFilterField::StatusClass => self.status_class = value,
            AuditFilterField::User => self.user = value,
            AuditFilterField::Method => self.method = value,
            AuditFilterField::ProgramCode => self.program_code = value,
            AuditFilterField::Path => self.path = value,
            AuditFilterField::Text => self.text = value,
        }
    }
}

/// One changed row of a data-change event
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DataChangeEntryDto {
    #[serde(default)]
    pub table: String,
    /// `INSERT` / `UPDATE` / `DELETE`
    #[serde(default)]
    pub op: String,
    #[serde(default)]
    pub row_id: Option<String>,
    #[serde(default)]
    pub columns: Vec<String>,
    #[serde(default)]
    pub old: Option<serde_json::Value>,
    #[serde(default)]
    pub new: Option<serde_json::Value>,
}
