//! Audit log port: records what users did (pages opened, buttons pressed, API calls with their
//! request / response) and searches it again. Implemented over Quickwit by `infrastructure`.
//!
//! Index `audit-logs-v1`; one document per [`AuditEvent`]. Recording must never block or fail a
//! user request: callers fire and forget, implementations buffer and drop on trouble.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Name of the Quickwit index holding the audit events
pub const AUDIT_INDEX: &str = "audit-logs-v1";

/// Quickwit index fed from the Redpanda topic `report.requests` (Kafka source, see
/// `make event-indexes`)
pub const EVENT_REPORT_INDEX: &str = "events-report-requests-v1";
/// Quickwit index fed from the Redpanda topic `notifications`
pub const EVENT_NOTIFICATION_INDEX: &str = "events-notifications-v1";

/// Kinds of audit events
pub mod audit_kind {
    /// A page (route) was opened in the web app
    pub const PAGE_VIEW: &str = "page_view";
    /// A button / link was pressed in the web app
    pub const CLICK: &str = "click";
    /// A business process was run in the web app (save, delete, approve ...)
    pub const ACTION: &str = "action";
    /// An API call, logged by the server (system log)
    pub const API: &str = "api";
    /// A row was inserted, updated or deleted in the database (captured by a trigger, see
    /// `al_data_change`): `changes` holds the rows before / after
    pub const DATA_CHANGE: &str = "data_change";
    /// A report job request that went through Redpanda (`report.requests`), kept in its own
    /// Quickwit index [`EVENT_REPORT_INDEX`]
    pub const EVENT_REPORT_REQUEST: &str = "event_report_request";
    /// A notification that went through Redpanda (`notifications`), kept in
    /// [`EVENT_NOTIFICATION_INDEX`]
    pub const EVENT_NOTIFICATION: &str = "event_notification";
    /// Search-only group: every event reported by the web app (page views, clicks, actions)
    pub const UI: &str = "ui";
    /// The kinds the group [`UI`] stands for
    pub const UI_KINDS: [&str; 3] = [PAGE_VIEW, CLICK, ACTION];
}

/// One audit event. Optional fields are absent for kinds they do not apply to
/// (e.g. `status` for a click, `element` for an API call).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AuditEvent {
    /// UUIDv7
    pub event_id: String,
    /// RFC 3339, UTC
    pub timestamp: String,
    /// See [`audit_kind`]
    pub kind: String,

    // Who
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub session_id: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    /// `th` / `en`, from `Accept-Language`
    pub language: Option<String>,

    // What (API call)
    pub method: Option<String>,
    /// Request path without the query string
    pub path: Option<String>,
    pub query: Option<String>,
    /// Catalogue program the path belongs to, e.g. `DBPG004`
    pub program_code: Option<String>,
    /// `VIEW` / `CREATE` / `UPDATE` / `DELETE` / `LOGIN` ... derived from the method and path
    pub action: Option<String>,
    pub status: Option<u16>,
    pub duration_ms: Option<u64>,
    /// Header name -> value; secrets (cookies, authorization, API keys) are replaced by `***`
    pub request_headers: Option<Value>,
    pub response_headers: Option<Value>,
    /// Bodies as JSON text (CBOR bodies are converted), secrets masked, cut at a size limit
    pub request_body: Option<String>,
    pub response_body: Option<String>,

    // What (web app)
    /// Route the user was on, e.g. `/db/dbpg004`
    pub page: Option<String>,
    /// What was pressed: tag + id / data attribute
    pub element: Option<String>,
    /// Visible text / aria-label of the element
    pub label: Option<String>,
    /// Free detail (process name, row id ...)
    pub detail: Option<String>,

    // What (data change: ONE event per API request, however many rows it touched; `action` is
    // INSERT / UPDATE / DELETE, or CHANGE when the request did several kinds, `detail` the
    // per-table summary)
    /// Tables touched, comma separated, e.g. `db_branch, db_branch_label`
    pub table_name: Option<String>,
    /// JSON text: one entry per changed row `{table, op, row_id, columns, old, new}`, rows before /
    /// after as objects, secrets masked
    pub changes: Option<String>,
    /// `event_id` of the API call (`kind=api`) that caused the change
    pub request_id: Option<String>,
}

/// Search filter; every set field narrows the result
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AuditSearch {
    /// RFC 3339, inclusive
    pub from: Option<String>,
    /// RFC 3339, exclusive
    pub to: Option<String>,
    pub kind: Option<String>,
    /// Matches user id, username or email exactly
    pub user: Option<String>,
    pub program_code: Option<String>,
    pub method: Option<String>,
    /// Exact status (`404`) ...
    pub status: Option<u16>,
    /// ... or a class: 2, 3, 4, 5 (`4xx`)
    pub status_class: Option<u8>,
    /// Substring of the request path
    pub path: Option<String>,
    /// Free text over the whole event (payloads, labels ...)
    pub text: Option<String>,
    /// 1-based
    pub page: u64,
    pub limit: u64,
}

/// One page of events, newest first
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AuditPage {
    pub items: Vec<AuditEvent>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

/// One slice of an export: the events of the filter, newest first, in store order. `cursor` is an
/// opaque token to pass back for the next slice; `None` means the export is complete.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AuditExportChunk {
    pub items: Vec<AuditEvent>,
    pub cursor: Option<String>,
}

/// Failure of the audit store
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuditError {
    /// The store (Quickwit) cannot be reached or answered with an error
    Unavailable(String),
    /// The search filter could not be turned into a query
    InvalidQuery(String),
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditError::Unavailable(m) => write!(f, "audit store unavailable: {m}"),
            AuditError::InvalidQuery(m) => write!(f, "invalid audit query: {m}"),
        }
    }
}

impl std::error::Error for AuditError {}

#[async_trait]
pub trait AuditLogPort: Send + Sync {
    /// Queues events for storage and returns immediately; never blocks on the store and never
    /// fails the caller (events are dropped with a warning when the store is down)
    fn record(&self, events: Vec<AuditEvent>);

    /// Searches events, newest first
    async fn search(&self, filter: AuditSearch) -> Result<AuditPage, AuditError>;

    /// One event by id
    async fn get(&self, event_id: &str) -> Result<Option<AuditEvent>, AuditError>;

    /// `true` when the store answers (used by the page to explain an empty result)
    async fn is_available(&self) -> bool;

    /// Next slice of an unbounded export of everything `filter` matches (`page` / `limit` are
    /// ignored). Start with `cursor = None`, then pass back the returned cursor until it is
    /// `None`. Memory stays bounded however many events match. Stores without export return
    /// nothing.
    async fn export_chunk(
        &self,
        _filter: AuditSearch,
        _cursor: Option<String>,
    ) -> Result<AuditExportChunk, AuditError> {
        Ok(AuditExportChunk::default())
    }
}

/// Audit log that stores nothing (audit disabled, tests): recording is ignored, searches are
/// empty and the store reports as unavailable
#[derive(Clone, Copy, Debug, Default)]
pub struct NoopAuditLog;

#[async_trait]
impl AuditLogPort for NoopAuditLog {
    fn record(&self, _events: Vec<AuditEvent>) {}

    async fn search(&self, filter: AuditSearch) -> Result<AuditPage, AuditError> {
        Ok(AuditPage { items: Vec::new(), total: 0, page: filter.page, limit: filter.limit })
    }

    async fn get(&self, _event_id: &str) -> Result<Option<AuditEvent>, AuditError> {
        Ok(None)
    }

    async fn is_available(&self) -> bool {
        false
    }
}
