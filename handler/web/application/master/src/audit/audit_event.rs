//! Wire shape of one client audit event.

use serde::Serialize;

pub const KIND_PAGE_VIEW: &str = "page_view";
pub const KIND_CLICK: &str = "click";
pub const KIND_ACTION: &str = "action";

/// One event of the `events` array of `POST /api/v1/audit/events`
#[derive(Clone, Debug, Serialize)]
pub struct AuditEventDto {
    pub kind: &'static str,
    pub page: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Request body of the batch call
#[derive(Clone, Debug, Serialize)]
pub struct AuditEventBatchDto {
    pub events: Vec<AuditEventDto>,
}
