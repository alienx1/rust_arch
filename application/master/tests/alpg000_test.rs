//! Tests for ALPG000 (Audit Log): filter normalization, client-event rules and the executor over a
//! mock audit store. Located at `application/master/tests/alpg000_test.rs`.

use application_core::errors::EventError;
use application_core::traits::{
    AuditError, AuditEvent, AuditLogPort, AuditPage, AuditSearch, EventService, REPORT_REQUESTS_TOPIC,
    ReportJobMessage,
};
use application_master::features::al::alpg000::{
    Alpg000CommandExecutor, Alpg000Commands, Alpg000Error, AuditLogFilterQuery, ClientActor,
    ClientEventCommand, normalize_client_events, normalize_search,
};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

/// Records what it is given and answers with canned data (or an error when `fail` is set)
#[derive(Default)]
struct MockStore {
    recorded: Mutex<Vec<AuditEvent>>,
    searched: Mutex<Vec<AuditSearch>>,
    fail: Mutex<Option<AuditError>>,
}

#[async_trait]
impl AuditLogPort for MockStore {
    fn record(&self, events: Vec<AuditEvent>) {
        self.recorded.lock().unwrap().extend(events);
    }
    async fn search(&self, filter: AuditSearch) -> Result<AuditPage, AuditError> {
        if let Some(e) = self.fail.lock().unwrap().clone() {
            return Err(e);
        }
        let (page, limit) = (filter.page, filter.limit);
        self.searched.lock().unwrap().push(filter);
        Ok(AuditPage {
            items: vec![AuditEvent {
                event_id: "e1".into(),
                kind: "api".into(),
                method: Some("GET".into()),
                request_body: Some("{}".into()),
                ..AuditEvent::default()
            }],
            total: 1,
            page,
            limit,
        })
    }
    async fn get(&self, id: &str) -> Result<Option<AuditEvent>, AuditError> {
        Ok((id == "e1").then(|| AuditEvent {
            event_id: "e1".into(),
            request_body: Some("{\"a\":1}".into()),
            ..AuditEvent::default()
        }))
    }
    async fn is_available(&self) -> bool {
        self.fail.lock().unwrap().is_none()
    }
}

fn query() -> AuditLogFilterQuery {
    AuditLogFilterQuery::default()
}

fn event(kind: &str) -> ClientEventCommand {
    ClientEventCommand { kind: kind.into(), page: Some("/db/dbpg004".into()), ..ClientEventCommand::default() }
}

#[test]
fn test_filter_defaults_and_bounds() {
    let f = normalize_search(query()).unwrap();
    assert_eq!((f.page, f.limit), (1, 25));
    let f = normalize_search(AuditLogFilterQuery { page: Some(0), limit: Some(10_000), ..query() }).unwrap();
    assert_eq!((f.page, f.limit), (1, 200));
}

#[test]
fn test_filter_values_are_cleaned_and_validated() {
    let f = normalize_search(AuditLogFilterQuery {
        kind: Some(" API ".into()),
        method: Some("post".into()),
        user: Some("  ".into()),
        ..query()
    })
    .unwrap();
    assert_eq!((f.kind.as_deref(), f.method.as_deref(), f.user), (Some("api"), Some("POST"), None));
    for bad in [
        AuditLogFilterQuery { kind: Some("nope".into()), ..query() },
        AuditLogFilterQuery { status_class: Some(9), ..query() },
        AuditLogFilterQuery { from: Some("yesterday".into()), ..query() },
        AuditLogFilterQuery { text: Some("x".repeat(201)), ..query() },
    ] {
        assert!(matches!(normalize_search(bad), Err(Alpg000Error::Validation(_))));
    }
}

#[test]
fn test_client_events_keep_only_ui_kinds_and_get_server_fields() {
    let actor = ClientActor { username: Some("somchai".into()), ..ClientActor::default() };
    let out = normalize_client_events(&actor, vec![event("click"), event("API"), event("page_view")]).unwrap();
    assert_eq!(out.len(), 2, "an `api` event can only come from the server");
    assert!(out.iter().all(|e| e.username.as_deref() == Some("somchai") && !e.timestamp.is_empty()));
    assert_ne!(out[0].event_id, out[1].event_id);
}

#[test]
fn test_more_than_fifty_events_is_rejected() {
    let actor = ClientActor::default();
    assert!(normalize_client_events(&actor, (0..50).map(|_| event("click")).collect()).is_ok());
    assert!(matches!(
        normalize_client_events(&actor, (0..51).map(|_| event("click")).collect()),
        Err(Alpg000Error::Validation(_))
    ));
}

#[tokio::test]
async fn test_executor_records_only_valid_events() {
    let store = Arc::new(MockStore::default());
    let exec = Alpg000CommandExecutor::new(store.clone());
    let n = exec
        .record_client_events(ClientActor::default(), vec![event("click"), event("api")])
        .await
        .unwrap();
    assert_eq!(n, 1);
    assert_eq!(store.recorded.lock().unwrap().len(), 1);

    let n = exec.record_client_events(ClientActor::default(), vec![event("api")]).await.unwrap();
    assert_eq!(n, 0);
    assert_eq!(store.recorded.lock().unwrap().len(), 1, "nothing is queued for an empty batch");
}

#[tokio::test]
async fn test_search_passes_the_normalized_filter_and_shapes_rows() {
    let store = Arc::new(MockStore::default());
    let exec = Alpg000CommandExecutor::new(store.clone());
    let page = exec
        .search_logs(AuditLogFilterQuery { program_code: Some("dbpg004".into()), limit: Some(500), ..query() })
        .await
        .unwrap();
    assert_eq!((page.total, page.page, page.limit, page.items.len()), (1, 1, 200, 1));
    assert_eq!(page.items[0].event_id, "e1");
    let seen = store.searched.lock().unwrap();
    assert_eq!(seen[0].program_code.as_deref(), Some("DBPG004"));
    assert_eq!(seen[0].limit, 200);
}

#[tokio::test]
async fn test_store_errors_are_mapped() {
    let store = Arc::new(MockStore::default());
    let exec = Alpg000CommandExecutor::new(store.clone());

    *store.fail.lock().unwrap() = Some(AuditError::Unavailable("down".into()));
    assert!(matches!(exec.search_logs(query()).await, Err(Alpg000Error::Unavailable(_))));
    assert!(!exec.status().await.available);

    *store.fail.lock().unwrap() = Some(AuditError::InvalidQuery("bad".into()));
    assert!(matches!(exec.search_logs(query()).await, Err(Alpg000Error::Validation(_))));

    *store.fail.lock().unwrap() = None;
    assert!(exec.status().await.available);
}

#[tokio::test]
async fn test_get_log() {
    let exec = Alpg000CommandExecutor::new(Arc::new(MockStore::default()));
    let found = exec.get_log("e1").await.unwrap().unwrap();
    assert_eq!(found.request_body.as_deref(), Some("{\"a\":1}"));
    assert!(exec.get_log("nope").await.unwrap().is_none());
    assert!(matches!(exec.get_log("  ").await, Err(Alpg000Error::Validation(_))));
}

#[test]
fn test_ui_is_an_accepted_kind_group() {
    let q = AuditLogFilterQuery { kind: Some("UI".to_string()), ..Default::default() };
    assert_eq!(normalize_search(q).unwrap().kind.as_deref(), Some("ui"));
}


/// Remembers what is published; `fail` makes the broker refuse
#[derive(Default)]
struct MockEvents {
    published: Mutex<Vec<(String, Option<String>, Vec<u8>)>>,
    fail: bool,
}

#[async_trait]
impl EventService for MockEvents {
    async fn publish(&self, topic: &str, key: Option<&str>, payload: &[u8]) -> Result<(), EventError> {
        if self.fail {
            return Err(EventError::PublishFailed { topic: topic.into(), reason: "broker down".into() });
        }
        self.published.lock().unwrap().push((topic.into(), key.map(str::to_string), payload.to_vec()));
        Ok(())
    }
}

#[tokio::test]
async fn test_request_export_publishes_the_normalized_filter_keyed_by_job_id() {
    let events = Arc::new(MockEvents::default());
    let exec = Alpg000CommandExecutor::new(Arc::new(MockStore::default())).with_events(events.clone());
    let filter = AuditLogFilterQuery { kind: Some(" API ".into()), program_code: Some("dbpg004".into()), ..query() };
    let queued = exec.request_export("somchai", filter).await.unwrap();

    let published = events.published.lock().unwrap();
    assert_eq!(published.len(), 1);
    let (topic, key, payload) = &published[0];
    assert_eq!(topic, REPORT_REQUESTS_TOPIC);
    assert_eq!(key.as_deref(), Some(queued.job_id.as_str()));
    let message: ReportJobMessage = serde_json::from_slice(payload).unwrap();
    assert_eq!(message.job_id, queued.job_id);
    assert_eq!(message.requested_by, "somchai");
    assert_eq!(message.report_kind, "audit_log_csv");
    assert_eq!(message.filter.kind.as_deref(), Some("api"));
    assert_eq!(message.filter.program_code.as_deref(), Some("DBPG004"));
}

#[tokio::test]
async fn test_request_export_rejects_a_bad_filter_and_reports_a_down_queue() {
    let events = Arc::new(MockEvents::default());
    let exec = Alpg000CommandExecutor::new(Arc::new(MockStore::default())).with_events(events.clone());
    let bad = AuditLogFilterQuery { kind: Some("nope".into()), ..query() };
    assert!(matches!(exec.request_export("somchai", bad).await, Err(Alpg000Error::Validation(_))));
    assert!(matches!(exec.request_export("  ", query()).await, Err(Alpg000Error::Validation(_))));
    assert!(events.published.lock().unwrap().is_empty(), "nothing is queued for a refused request");

    let down = Alpg000CommandExecutor::new(Arc::new(MockStore::default()))
        .with_events(Arc::new(MockEvents { fail: true, ..Default::default() }));
    assert!(matches!(down.request_export("somchai", query()).await, Err(Alpg000Error::Unavailable(_))));
    let unconfigured = Alpg000CommandExecutor::new(Arc::new(MockStore::default()));
    assert!(matches!(unconfigured.request_export("somchai", query()).await, Err(Alpg000Error::Unavailable(_))));
}
