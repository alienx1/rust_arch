use infrastructure::quickwit::{DataChangeRow, changes_to_event, group_changes};
use serde_json::Value;

fn row(table: &str, op: &str, old: Option<&str>, new: Option<&str>) -> DataChangeRow {
    DataChangeRow {
        id: format!("id-{table}-{op}"),
        changed_at: "2026-10-11T08:00:00.123Z".into(),
        txid: "10".into(),
        table_name: table.into(),
        op: op.into(),
        row_id: Some("b1".into()),
        actor_id: "kratos-1".into(),
        actor_name: Some("admin".into()),
        request_id: Some("req-1".into()),
        old_data: old.map(Into::into),
        new_data: new.map(Into::into),
    }
}

fn changes(e: &application_core::traits::AuditEvent) -> Vec<Value> {
    serde_json::from_str::<Value>(e.changes.as_deref().unwrap()).unwrap().as_array().unwrap().clone()
}

#[test]
fn update_lists_changed_columns_without_row_version() {
    let e = changes_to_event(&[row(
        "db_branch",
        "UPDATE",
        Some(r#"{"id":"b1","code":"A","row_version":1}"#),
        Some(r#"{"id":"b1","code":"B","row_version":2}"#),
    )])
    .unwrap();
    assert_eq!(e.kind, "data_change");
    assert_eq!(e.action.as_deref(), Some("UPDATE"));
    assert_eq!(e.detail.as_deref(), Some("db_branch UPDATE"));
    assert_eq!(e.table_name.as_deref(), Some("db_branch"));
    assert_eq!(e.user_id.as_deref(), Some("kratos-1"));
    assert_eq!(e.request_id.as_deref(), Some("req-1"));
    let c = changes(&e);
    assert_eq!(c[0]["columns"], serde_json::json!(["code"]));
    assert_eq!(c[0]["old"]["code"], "A");
    assert_eq!(c[0]["new"]["code"], "B");
}

#[test]
fn version_only_updates_are_dropped() {
    let noise = row("db_branch", "UPDATE", Some(r#"{"id":"b1","row_version":1}"#), Some(r#"{"id":"b1","row_version":2}"#));
    assert!(changes_to_event(&[noise.clone()]).is_none());
    let real = row("db_branch_label", "UPDATE", Some(r#"{"label":"a","row_version":1}"#), Some(r#"{"label":"b","row_version":2}"#));
    let e = changes_to_event(&[noise, real]).unwrap();
    assert_eq!(changes(&e).len(), 1);
    assert_eq!(e.table_name.as_deref(), Some("db_branch_label"));
}

#[test]
fn one_request_is_one_event_with_a_summary() {
    let rows = vec![
        row("db_branch", "UPDATE", Some(r#"{"a":1}"#), Some(r#"{"a":2}"#)),
        row("db_branch_label", "INSERT", None, Some(r#"{"label":"x"}"#)),
        row("db_branch_label", "INSERT", None, Some(r#"{"label":"y"}"#)),
    ];
    let mut rows = rows;
    for (i, r) in rows.iter_mut().enumerate() {
        r.id = format!("id-{i}");
        r.changed_at = format!("2026-10-11T08:00:00.12{i}Z");
    }
    let groups = group_changes(rows);
    assert_eq!(groups.len(), 1);
    let e = changes_to_event(&groups[0]).unwrap();
    assert_eq!(e.action.as_deref(), Some("CHANGE"));
    assert_eq!(e.detail.as_deref(), Some("db_branch UPDATE, db_branch_label INSERT ×2"));
    assert_eq!(e.table_name.as_deref(), Some("db_branch, db_branch_label"));
    assert_eq!(changes(&e).len(), 3);
}

#[test]
fn rows_of_different_requests_stay_apart_and_missing_request_groups_by_transaction() {
    let mut a = row("t1", "DELETE", Some(r#"{"id":"1"}"#), None);
    let mut b = row("t2", "DELETE", Some(r#"{"id":"2"}"#), None);
    b.request_id = Some("req-2".into());
    assert_eq!(group_changes(vec![a.clone(), b]).len(), 2);
    a.request_id = None;
    let mut c = a.clone();
    c.id = "other".into();
    assert_eq!(group_changes(vec![a, c]).len(), 1);
}

#[test]
fn secrets_are_masked_but_still_count_as_changed() {
    let e = changes_to_event(&[row(
        "su_user",
        "UPDATE",
        Some(r#"{"id":"u","password_hash":"old-hash"}"#),
        Some(r#"{"id":"u","password_hash":"new-hash"}"#),
    )])
    .unwrap();
    let c = changes(&e);
    assert_eq!(c[0]["columns"], serde_json::json!(["password_hash"]));
    let text = e.changes.unwrap();
    assert!(text.contains("***") && !text.contains("old-hash") && !text.contains("new-hash"));
}
