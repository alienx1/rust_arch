//! Pure parts of the Quickwit audit adapter: query builder, escaping, time range, paging,
//! document conversion, config. The `#[ignore]` test at the bottom needs a live Quickwit.

use application_core::traits::{AuditEvent, AuditLogPort, AuditSearch, audit_kind};
use infrastructure::services::quickwit::audit_flusher::to_ndjson;
use infrastructure::services::quickwit::audit_query::quote_term;
use infrastructure::services::quickwit::{
    AuditTimeRange, QuickwitAuditLog, QuickwitConfig, audit_time_range, build_audit_query,
    doc_to_event, event_to_doc, export_search_body, page_window,
};
use serde_json::json;

#[test]
fn empty_filter_matches_everything() {
    assert_eq!(build_audit_query(&AuditSearch::default()), "*");
    let blank = AuditSearch { kind: Some("  ".into()), text: Some("".into()), ..Default::default() };
    assert_eq!(build_audit_query(&blank), "*");
}

#[test]
fn exact_filters_and_status() {
    let f = AuditSearch {
        kind: Some("api".into()),
        program_code: Some("DBPG004".into()),
        method: Some("post".into()),
        status: Some(404),
        status_class: Some(5),
        ..Default::default()
    };
    assert_eq!(
        build_audit_query(&f),
        r#"kind:"api" AND program_code:"DBPG004" AND method:"POST" AND status:404"#
    );
    let class = AuditSearch { status_class: Some(4), ..Default::default() };
    assert_eq!(build_audit_query(&class), "status:[400 TO 499]");
    let bad = AuditSearch { status_class: Some(9), ..Default::default() };
    assert_eq!(build_audit_query(&bad), "*");
}

#[test]
fn user_matches_id_username_or_email() {
    let f = AuditSearch { user: Some("alice".into()), ..Default::default() };
    assert_eq!(
        build_audit_query(&f),
        r#"(user_id:"alice" OR username:"alice" OR email:"alice")"#
    );
}

#[test]
fn user_input_cannot_inject_query_syntax() {
    assert_eq!(quote_term(r#"a" OR kind:"api"#), r#""a\" OR kind:\"api""#);
    assert_eq!(quote_term(r"a\b"), r#""a\\b""#);
    let f = AuditSearch { user: Some(r#"x" OR user_id:*"#.into()), ..Default::default() };
    let q = build_audit_query(&f);
    assert!(q.starts_with(r#"(user_id:"x\" OR user_id:*""#), "{q}");
}

#[test]
fn path_is_a_token_match() {
    let one = AuditSearch { path: Some("DBPG004".into()), ..Default::default() };
    assert_eq!(build_audit_query(&one), "path:dbpg004*");
    let many = AuditSearch { path: Some("/api/v1/db/dbpg004/x-y".into()), ..Default::default() };
    assert_eq!(build_audit_query(&many), r#"path:"api v1 db dbpg004 x y""#);
    let junk = AuditSearch { path: Some("///".into()), ..Default::default() };
    assert_eq!(build_audit_query(&junk), "*");
}

#[test]
fn free_text_words_are_quoted_and_sanitised() {
    // Words split exactly like the index's default tokenizer (non-alphanumerics, Thai marks too)
    let f = AuditSearch { text: Some(r#"Hello AND kind:api "x" ค้นหา"#.into()), ..Default::default() };
    assert_eq!(
        build_audit_query(&f),
        r#""hello" AND "and" AND "kind" AND "api" AND "x" AND "ค" AND "นหา""#
    );
}

#[test]
fn time_range_is_unix_seconds_with_exclusive_end() {
    let f = AuditSearch {
        from: Some("2026-01-01T00:00:00Z".into()),
        to: Some("2026-01-01T07:00:00.500+07:00".into()),
        ..Default::default()
    };
    // 2026-01-01T00:00:00Z = 1767225600; `to` is 00:00:00.5Z -> rounded up
    assert_eq!(
        audit_time_range(&f).unwrap(),
        AuditTimeRange { start: Some(1_767_225_600), end: Some(1_767_225_601) }
    );
    assert_eq!(audit_time_range(&AuditSearch::default()).unwrap(), AuditTimeRange::default());
    let bad = AuditSearch { from: Some("yesterday".into()), ..Default::default() };
    assert!(audit_time_range(&bad).is_err());
}

#[test]
fn paging_window() {
    let f = |page, limit| AuditSearch { page, limit, ..Default::default() };
    assert_eq!(page_window(&f(1, 25)), (25, 0));
    assert_eq!(page_window(&f(3, 25)), (25, 50));
    assert_eq!(page_window(&f(0, 0)), (50, 0));
    assert_eq!(page_window(&f(2, 10_000)), (200, 200));
}

fn sample() -> AuditEvent {
    AuditEvent {
        event_id: "019a0000-0000-7000-8000-000000000001".into(),
        timestamp: "2026-10-09T10:00:00.123Z".into(),
        kind: audit_kind::API.into(),
        username: Some("alice".into()),
        method: Some("GET".into()),
        path: Some("/api/v1/db/dbpg004/x-y".into()),
        status: Some(200),
        request_headers: Some(json!({"accept": "application/json"})),
        ..Default::default()
    }
}

#[test]
fn document_has_path_raw_and_no_nulls_and_round_trips() {
    let ev = sample();
    let doc = event_to_doc(&ev);
    assert_eq!(doc["path_raw"], "/api/v1/db/dbpg004/x-y");
    assert_eq!(doc["path"], "/api/v1/db/dbpg004/x-y");
    assert!(doc.get("user_id").is_none(), "null fields are not ingested");
    assert_eq!(doc["status"], 200);
    assert_eq!(doc_to_event(doc).unwrap(), ev);

    let mut no_path = sample();
    no_path.path = None;
    assert!(event_to_doc(&no_path).get("path_raw").is_none());
}

#[test]
fn ndjson_is_one_document_per_line() {
    let body = to_ndjson(&[sample(), sample()]);
    assert_eq!(body.lines().count(), 2);
    assert!(body.ends_with('\n'));
    for line in body.lines() {
        assert!(serde_json::from_str::<serde_json::Value>(line).is_ok());
    }
}

#[test]
fn config_defaults_and_flags() {
    let d = QuickwitConfig::from_values(None, None);
    assert_eq!(d, QuickwitConfig { url: "http://localhost:7280".into(), enabled: true });
    let c = QuickwitConfig::from_values(Some(" http://qw:7280/ "), Some("FALSE"));
    assert_eq!(c, QuickwitConfig { url: "http://qw:7280".into(), enabled: false });
    assert!(QuickwitConfig::from_values(None, Some("true")).enabled);
}

/// Needs Quickwit on QUICKWIT_URL (default http://localhost:7280):
/// `cargo test -p infrastructure --test quickwit_audit_query_test -- --ignored`
#[tokio::test]
#[ignore]
async fn live_round_trip() {
    let log = QuickwitAuditLog::start(QuickwitConfig::from_env());
    assert!(log.is_available().await, "Quickwit is not reachable");

    let marker = format!("rt{}", uuid::Uuid::now_v7().simple());
    let mut ev = sample();
    ev.event_id = uuid::Uuid::now_v7().to_string();
    ev.timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    ev.username = Some(marker.clone());
    ev.status = Some(404);
    ev.request_body = Some(format!("{{\"note\":\"{marker}\"}}"));
    log.record(vec![ev.clone()]);

    // flusher: 1 s tick + commit_timeout 5 s
    let by_user = AuditSearch { user: Some(marker.clone()), page: 1, limit: 10, ..Default::default() };
    let mut found = None;
    for _ in 0..30 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let page = log.search(by_user.clone()).await.expect("search");
        if page.total > 0 {
            found = Some(page);
            break;
        }
    }
    let page = found.expect("event not searchable within 30 s");
    assert_eq!(page.total, 1);
    assert_eq!(page.items[0], ev);

    let by_class = AuditSearch { user: Some(marker.clone()), status_class: Some(4), path: Some("dbpg004".into()), text: Some(marker.clone()), page: 1, limit: 10, ..Default::default() };
    assert_eq!(log.search(by_class).await.unwrap().total, 1);
    let miss = AuditSearch { user: Some(marker.clone()), status_class: Some(2), page: 1, limit: 10, ..Default::default() };
    assert_eq!(log.search(miss).await.unwrap().total, 0);

    assert_eq!(log.get(&ev.event_id).await.unwrap(), Some(ev));
    assert_eq!(log.get("does-not-exist").await.unwrap(), None);
}

#[test]
fn test_ui_kind_expands_to_the_three_web_app_kinds() {
    let q = build_audit_query(&AuditSearch { kind: Some("ui".to_string()), ..Default::default() });
    assert_eq!(q, r#"(kind:"page_view" OR kind:"click" OR kind:"action")"#);
}

#[test]
fn test_export_body_has_query_time_range_and_newest_first_paging() {
    let f = AuditSearch {
        kind: Some("api".into()),
        from: Some("2026-10-01T00:00:00Z".into()),
        to: Some("2026-10-02T00:00:00.5Z".into()),
        page: 7,
        limit: 3,
        ..Default::default()
    };
    let body = export_search_body(&f).unwrap();
    assert_eq!(body["query"]["bool"]["must"][0]["query_string"]["query"], r#"kind:"api""#);
    let range = &body["query"]["bool"]["filter"][0]["range"]["timestamp"];
    assert_eq!(range["gte"], "2026-10-01T00:00:00Z");
    // a sub-second `to` is rounded up so it does not cut its own second
    assert_eq!(range["lt"], "2026-10-02T00:00:01Z");
    assert_eq!(body["sort"][0]["timestamp"]["order"], "desc");
    assert_eq!(body["size"], 5000, "page / limit of the list do not apply to an export");

    let open = export_search_body(&AuditSearch::default()).unwrap();
    assert_eq!(open["query"]["bool"]["must"][0]["query_string"]["query"], "*");
    assert!(open["query"]["bool"].get("filter").is_none());

    let bad = AuditSearch { from: Some("yesterday".into()), ..Default::default() };
    assert!(export_search_body(&bad).is_err());
}

/// Exports more than one scroll page (5,000 events) of a marker user and checks every event comes
/// back exactly once. Leaves its ~11k events in the index (delete by the marker if unwanted).
/// Needs Quickwit: `cargo test -p infrastructure --test quickwit_audit_query_test -- --ignored`
#[tokio::test]
#[ignore]
async fn live_export_streams_every_event_across_scroll_pages() {
    let log = QuickwitAuditLog::start(QuickwitConfig::from_env());
    assert!(log.is_available().await, "Quickwit is not reachable");
    let marker = format!("ex{}", uuid::Uuid::now_v7().simple());
    let total = 11_000usize;
    let base = chrono::Utc::now();
    for batch in 0..(total / 1000) {
        let events = (0..1000)
            .map(|i| {
                let n = batch * 1000 + i;
                let mut ev = sample();
                ev.event_id = uuid::Uuid::now_v7().to_string();
                ev.timestamp = (base + chrono::Duration::milliseconds(n as i64))
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                ev.username = Some(marker.clone());
                ev
            })
            .collect();
        log.record(events);
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }
    let filter = AuditSearch { user: Some(marker.clone()), ..Default::default() };
    for _ in 0..60 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        if log.search(AuditSearch { page: 1, limit: 1, ..filter.clone() }).await.unwrap().total as usize >= total {
            break;
        }
    }
    let (mut seen, mut slices, mut cursor) = (std::collections::HashSet::new(), 0, None);
    loop {
        let chunk = log.export_chunk(filter.clone(), cursor).await.expect("export");
        slices += 1;
        chunk.items.iter().for_each(|e| {
            assert!(seen.insert(e.event_id.clone()), "duplicate {}", e.event_id);
        });
        match chunk.cursor {
            Some(c) => cursor = Some(c),
            None => break,
        }
    }
    assert_eq!(seen.len(), total);
    assert!(slices >= 3, "expected several scroll pages, got {slices}");
}
