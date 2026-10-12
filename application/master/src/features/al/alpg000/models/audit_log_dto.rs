//! ALPG000 audit rows: the list row (no headers / bodies) and the full event

use application_core::traits::AuditEvent;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// List row
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AuditLogSummaryDto {
    pub event_id: String,
    pub timestamp: String,
    pub kind: String,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub ip: Option<String>,
    pub method: Option<String>,
    pub path: Option<String>,
    pub program_code: Option<String>,
    pub action: Option<String>,
    pub status: Option<u16>,
    pub duration_ms: Option<u64>,
    pub page: Option<String>,
    pub element: Option<String>,
    pub label: Option<String>,
    pub detail: Option<String>,
    pub table_name: Option<String>,
}

/// Full event for the detail view
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AuditLogDto {
    pub event_id: String,
    pub timestamp: String,
    pub kind: String,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub session_id: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub language: Option<String>,
    pub method: Option<String>,
    pub path: Option<String>,
    pub query: Option<String>,
    pub program_code: Option<String>,
    pub action: Option<String>,
    pub status: Option<u16>,
    pub duration_ms: Option<u64>,
    /// Header name -> value (secrets masked)
    pub request_headers: Option<Value>,
    pub response_headers: Option<Value>,
    /// JSON text, masked and truncated
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub page: Option<String>,
    pub element: Option<String>,
    pub label: Option<String>,
    pub detail: Option<String>,
    /// Data changes (`kind=data_change`): tables touched, and per changed row the table, op, key,
    /// changed columns and the row before / after (JSON text)
    pub table_name: Option<String>,
    pub changes: Option<String>,
    /// `event_id` of the API call that made the change
    pub request_id: Option<String>,
}

impl From<AuditEvent> for AuditLogSummaryDto {
    fn from(e: AuditEvent) -> Self {
        Self {
            event_id: e.event_id,
            timestamp: e.timestamp,
            kind: e.kind,
            user_id: e.user_id,
            username: e.username,
            ip: e.ip,
            method: e.method,
            path: e.path,
            program_code: e.program_code,
            action: e.action,
            status: e.status,
            duration_ms: e.duration_ms,
            page: e.page,
            element: e.element,
            label: e.label,
            detail: e.detail,
            table_name: e.table_name,
        }
    }
}

impl From<AuditEvent> for AuditLogDto {
    fn from(e: AuditEvent) -> Self {
        Self {
            event_id: e.event_id,
            timestamp: e.timestamp,
            kind: e.kind,
            user_id: e.user_id,
            username: e.username,
            email: e.email,
            session_id: e.session_id,
            ip: e.ip,
            user_agent: e.user_agent,
            language: e.language,
            method: e.method,
            path: e.path,
            query: e.query,
            program_code: e.program_code,
            action: e.action,
            status: e.status,
            duration_ms: e.duration_ms,
            request_headers: e.request_headers,
            response_headers: e.response_headers,
            request_body: e.request_body,
            response_body: e.response_body,
            page: e.page,
            element: e.element,
            label: e.label,
            detail: e.detail,
            table_name: e.table_name,
            changes: e.changes,
            request_id: e.request_id,
        }
    }
}
