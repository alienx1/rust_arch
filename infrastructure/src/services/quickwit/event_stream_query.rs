//! Pure builders for the Redpanda event indexes (`events-report-requests-v1`,
//! `events-notifications-v1`, fed by Quickwit's Kafka source): which index a kind lives in, the
//! query for the filters that apply to them, and the mapping of a stored message to an
//! [`AuditEvent`] so ALPG000 lists them like any other event. No I/O here.

use super::audit_query::{non_empty, quote_term, words};
use application_core::traits::{
    AuditEvent, AuditSearch, EVENT_NOTIFICATION_INDEX, EVENT_REPORT_INDEX, audit_kind,
};
use serde_json::Value;

/// Index holding the events of `kind`, `None` for the kinds of the audit index
pub fn event_index_for(kind: &str) -> Option<&'static str> {
    match kind {
        audit_kind::EVENT_REPORT_REQUEST => Some(EVENT_REPORT_INDEX),
        audit_kind::EVENT_NOTIFICATION => Some(EVENT_NOTIFICATION_INDEX),
        _ => None,
    }
}

/// `true` for the kinds served from the event indexes
pub fn is_event_kind(kind: &str) -> bool {
    event_index_for(kind).is_some()
}

/// Query for an event index. Only the filters that exist on these messages apply: `user`
/// (requester / recipient, exact) and `text` (every word, over the default search fields).
pub fn build_event_query(kind: &str, filter: &AuditSearch) -> String {
    let user_field = if kind == audit_kind::EVENT_REPORT_REQUEST { "requested_by" } else { "recipient" };
    let mut parts: Vec<String> = Vec::new();
    if let Some(v) = non_empty(&filter.user) {
        parts.push(format!("{user_field}:{}", quote_term(v)));
    }
    if let Some(v) = non_empty(&filter.text) {
        for w in words(v) {
            parts.push(format!("\"{w}\""));
        }
    }
    if parts.is_empty() { "*".to_string() } else { parts.join(" AND ") }
}

fn text(doc: &Value, key: &str) -> Option<String> {
    doc.get(key).and_then(Value::as_str).map(str::to_string).filter(|s| !s.is_empty())
}

fn json_text(doc: &Value, key: &str) -> Option<String> {
    doc.get(key).filter(|v| !v.is_null()).map(|v| v.to_string())
}

/// Stored message of `kind` as an audit event (`None` when it has no id or time)
pub fn event_doc_to_audit(kind: &str, doc: &Value) -> Option<AuditEvent> {
    if kind == audit_kind::EVENT_REPORT_REQUEST {
        Some(AuditEvent {
            event_id: text(doc, "job_id")?,
            timestamp: text(doc, "requested_at")?,
            kind: kind.to_string(),
            user_id: text(doc, "requested_by"),
            username: text(doc, "requested_by"),
            action: text(doc, "report_kind"),
            label: Some("report.requests".to_string()),
            request_body: json_text(doc, "filter"),
            ..AuditEvent::default()
        })
    } else if kind == audit_kind::EVENT_NOTIFICATION {
        Some(AuditEvent {
            event_id: text(doc, "id")?,
            timestamp: text(doc, "created_at")?,
            kind: kind.to_string(),
            user_id: text(doc, "recipient"),
            username: text(doc, "recipient"),
            action: text(doc, "kind"),
            label: text(doc, "title"),
            detail: text(doc, "body"),
            page: text(doc, "link"),
            response_body: json_text(doc, "data"),
            ..AuditEvent::default()
        })
    } else {
        None
    }
}
