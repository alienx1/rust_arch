//! Event indexes (Redpanda -> Quickwit Kafka source): index per kind, query, message mapping

use application_core::traits::{AuditSearch, EVENT_NOTIFICATION_INDEX, EVENT_REPORT_INDEX, audit_kind};
use infrastructure::services::quickwit::{build_event_query, event_doc_to_audit, event_index_for, is_event_kind};
use serde_json::json;

#[test]
fn each_event_kind_has_its_own_index() {
    assert_eq!(event_index_for(audit_kind::EVENT_REPORT_REQUEST), Some(EVENT_REPORT_INDEX));
    assert_eq!(event_index_for(audit_kind::EVENT_NOTIFICATION), Some(EVENT_NOTIFICATION_INDEX));
    assert_eq!(event_index_for(audit_kind::API), None);
    assert!(is_event_kind(audit_kind::EVENT_NOTIFICATION));
    assert!(!is_event_kind("ui"));
}

#[test]
fn query_uses_the_user_field_of_the_index_and_quotes_input() {
    let f = AuditSearch { user: Some("a\" OR x:*".into()), ..AuditSearch::default() };
    assert_eq!(build_event_query(audit_kind::EVENT_NOTIFICATION, &f), "recipient:\"a\\\" OR x:*\"");
    assert!(build_event_query(audit_kind::EVENT_REPORT_REQUEST, &f).starts_with("requested_by:"));
    assert_eq!(build_event_query(audit_kind::EVENT_NOTIFICATION, &AuditSearch::default()), "*");
}

#[test]
fn text_words_are_and_ed_and_audit_only_filters_are_ignored() {
    let f = AuditSearch { text: Some("report ready".into()), method: Some("GET".into()), status: Some(500), ..AuditSearch::default() };
    assert_eq!(build_event_query(audit_kind::EVENT_NOTIFICATION, &f), "\"report\" AND \"ready\"");
}

#[test]
fn notification_message_maps_to_an_audit_event() {
    let doc = json!({"id":"n1","recipient":"u1","kind":"report_ready","title":"Done","body":"file","link":"/rp/rppg000","data":{"a":1},"created_at":"2026-10-12T05:21:24Z"});
    let e = event_doc_to_audit(audit_kind::EVENT_NOTIFICATION, &doc).unwrap();
    assert_eq!((e.event_id.as_str(), e.timestamp.as_str(), e.kind.as_str()), ("n1", "2026-10-12T05:21:24Z", "event_notification"));
    assert_eq!(e.user_id.as_deref(), Some("u1"));
    assert_eq!(e.action.as_deref(), Some("report_ready"));
    assert_eq!(e.label.as_deref(), Some("Done"));
    assert_eq!(e.page.as_deref(), Some("/rp/rppg000"));
    assert_eq!(e.response_body.as_deref(), Some("{\"a\":1}"));
}

#[test]
fn report_request_maps_and_a_message_without_id_or_time_is_skipped() {
    let doc = json!({"job_id":"j1","report_kind":"audit_log_csv","requested_by":"admin","requested_at":"2026-10-12T05:00:00Z","filter":{"kind":"api"}});
    let e = event_doc_to_audit(audit_kind::EVENT_REPORT_REQUEST, &doc).unwrap();
    assert_eq!(e.event_id, "j1");
    assert_eq!(e.action.as_deref(), Some("audit_log_csv"));
    assert_eq!(e.request_body.as_deref(), Some("{\"kind\":\"api\"}"));
    assert!(event_doc_to_audit(audit_kind::EVENT_REPORT_REQUEST, &json!({"report_kind":"x"})).is_none());
    assert!(event_doc_to_audit(audit_kind::API, &doc).is_none());
}
