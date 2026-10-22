//! Change notices: which requests are "a user changed data", and that the user who made one is
//! told through the notification port. Located at `infrastructure/tests/change_notice_test.rs`.

use std::sync::{Arc, Mutex};

use application_core::traits::{Notification, NotificationService, notification_kind};
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{post, put};
use axum::Router;
use infrastructure::middleware::{AuthenticatedUser, ChangeNoticeState, change_notice, describe_change};
use infrastructure::middleware::notice::{ChangeKind, ChangeNotice};
use tower::ServiceExt;

fn notice(method: &str, path: &str, status: u16) -> Option<ChangeNotice> {
    describe_change(method, path, status)
}

#[test]
fn test_a_write_on_a_program_api_is_a_change() {
    let created = notice("POST", "/api/v1/su/supg004/users", 200).unwrap();
    assert_eq!((created.program.as_str(), created.target.as_str(), created.kind), ("SUPG004", "users", ChangeKind::Created));
    assert_eq!(created.link, "/su/supg004", "the notice opens the program");

    let updated = notice("PUT", "/api/v1/master/db/dbpg003/employees/0194", 200).unwrap();
    assert_eq!((updated.program.as_str(), updated.target.as_str(), updated.kind), ("DBPG003", "employees", ChangeKind::Updated));
    assert_eq!(updated.link, "/db/dbpg003", "the service prefix is not part of the web path");

    assert_eq!(notice("DELETE", "/api/v1/db/dbpg002/branches/0194", 204).unwrap().kind, ChangeKind::Deleted);
    // An action on a record (POST below the collection) is an update, not a new row
    assert_eq!(notice("POST", "/api/v1/su/supg004/users/0194/password-mail", 200).unwrap().kind, ChangeKind::Updated);
    assert_eq!(notice("PUT", "/api/v1/su/supg006/config", 200).unwrap().target, "config");
}

#[test]
fn test_a_refused_or_failed_write_is_a_failure_with_its_status() {
    for status in [400, 403, 404, 412, 500] {
        let n = notice("PUT", "/api/v1/db/dbpg004/lookups/1", status).unwrap();
        assert_eq!((n.kind, n.status), (ChangeKind::Failed, status));
    }
    // No session: not a change and nobody to tell
    assert_eq!(notice("PUT", "/api/v1/db/dbpg004/lookups/1", 401), None);
}

#[test]
fn test_reads_and_quiet_programs_and_form_parts_are_not_noticed() {
    assert_eq!(notice("GET", "/api/v1/db/dbpg003/employees", 200), None);
    assert_eq!(notice("POST", "/api/v1/audit/events", 200), None, "client audit events are not data entry");
    assert_eq!(notice("POST", "/api/v1/al/alpg000/exports", 202), None, "exports are reported by the report worker");
    assert_eq!(notice("POST", "/api/v1/rp/rppg000/jobs/1/download", 200), None);
    assert_eq!(notice("PUT", "/api/v1/hooks/password-changed", 200), None, "hooks are not under a program");
    assert_eq!(notice("PUT", "/health", 200), None);
    // The form saves the record, then its contacts and picture: one notice for the save
    assert_eq!(notice("PUT", "/api/v1/db/dbpg003/employees/1/contacts", 200), None);
    assert_eq!(notice("PUT", "/api/v1/db/dbpg001/organizations/1/image", 200), None);
    assert_eq!(notice("DELETE", "/api/v1/db/dbpg001/companies/1/image", 204), None);
}

/// What reached the notification port: stored notifications and live-only ones, apart
#[derive(Default)]
struct Bell {
    stored: Mutex<Vec<(String, Notification)>>,
    live: Mutex<Vec<(String, Notification)>>,
}

#[async_trait]
impl NotificationService for Bell {
    async fn notify(&self, username: &str, notification: Notification) {
        self.stored.lock().unwrap().push((username.to_string(), notification));
    }
    async fn notify_live(&self, username: &str, notification: Notification) {
        self.live.lock().unwrap().push((username.to_string(), notification));
    }
}

fn user() -> AuthenticatedUser {
    AuthenticatedUser {
        identity_id: "i".into(),
        session_id: "s".into(),
        email: Some("somchai@arch.local".into()),
        traits: serde_json::json!({ "username": "somchai" }),
    }
}

/// A handler the way the session middleware leaves it: the user is on the response
async fn saved() -> impl IntoResponse {
    let mut response = StatusCode::OK.into_response();
    response.extensions_mut().insert(user());
    response
}

async fn refused() -> impl IntoResponse {
    let mut response = StatusCode::PRECONDITION_FAILED.into_response();
    response.extensions_mut().insert(user());
    response
}

fn app(bell: Arc<Bell>) -> Router {
    Router::new()
        .route("/api/v1/su/supg004/users/{id}", put(saved))
        .route("/api/v1/su/supg004/users", post(saved))
        .route("/api/v1/db/dbpg004/lookups/{id}", put(refused))
        .route("/api/v1/su/supg004/anonymous", post(|| async { StatusCode::OK }))
        .layer(axum::middleware::from_fn_with_state(ChangeNoticeState::new(Some(bell)), change_notice))
}

async fn request(app: &Router, method: &str, uri: &str) -> StatusCode {
    let req = Request::builder().method(method).uri(uri).body(Body::empty()).unwrap();
    app.clone().oneshot(req).await.unwrap().status()
}

async fn settle(bell: &Bell, expected: usize) -> Vec<(String, Notification)> {
    for _ in 0..50 {
        if bell.live.lock().unwrap().len() + bell.stored.lock().unwrap().len() >= expected {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let mut all = bell.live.lock().unwrap().clone();
    all.extend(bell.stored.lock().unwrap().clone());
    all
}

#[tokio::test]
async fn test_the_user_who_saved_is_notified_with_the_kind_and_where_to_look() {
    let bell = Arc::new(Bell::default());
    let app = app(bell.clone());
    assert_eq!(request(&app, "PUT", "/api/v1/su/supg004/users/1").await, StatusCode::OK);
    assert_eq!(request(&app, "POST", "/api/v1/su/supg004/users").await, StatusCode::OK);

    let sent = settle(&bell, 2).await;
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[0].0, "somchai");
    let kinds: Vec<&str> = sent.iter().map(|(_, n)| n.kind.as_str()).collect();
    assert!(kinds.contains(&notification_kind::DATA_UPDATED) && kinds.contains(&notification_kind::DATA_CREATED), "{kinds:?}");
    let n = &sent.iter().find(|(_, n)| n.kind == notification_kind::DATA_UPDATED).unwrap().1;
    assert_eq!(n.link.as_deref(), Some("/su/supg004"));
    assert_eq!((n.data["program"].as_str(), n.data["target"].as_str(), n.data["status"].as_u64()), (Some("SUPG004"), Some("users"), Some(200)));
}

#[tokio::test]
async fn test_a_failed_save_is_reported_as_a_failure() {
    let bell = Arc::new(Bell::default());
    let app = app(bell.clone());
    assert_eq!(request(&app, "PUT", "/api/v1/db/dbpg004/lookups/1").await, StatusCode::PRECONDITION_FAILED);
    let sent = settle(&bell, 1).await;
    assert_eq!(sent[0].1.kind, notification_kind::DATA_FAILED);
    assert_eq!(sent[0].1.data["status"], 412);
    assert_eq!((bell.stored.lock().unwrap().len(), bell.live.lock().unwrap().len()), (1, 0), "a failure is kept for the user to find");
}

#[tokio::test]
async fn test_a_success_is_live_only_and_stores_nothing() {
    let bell = Arc::new(Bell::default());
    let app = app(bell.clone());
    assert_eq!(request(&app, "PUT", "/api/v1/su/supg004/users/1").await, StatusCode::OK);
    assert_eq!(request(&app, "POST", "/api/v1/su/supg004/users").await, StatusCode::OK);
    settle(&bell, 2).await;
    assert_eq!(bell.live.lock().unwrap().len(), 2, "pushed to the open sessions");
    assert!(bell.stored.lock().unwrap().is_empty(), "no row, no index: the audit trail has the change");
}

#[tokio::test]
async fn test_a_request_without_a_user_tells_nobody() {
    let bell = Arc::new(Bell::default());
    let app = app(bell.clone());
    assert_eq!(request(&app, "POST", "/api/v1/su/supg004/anonymous").await, StatusCode::OK);
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert!(bell.live.lock().unwrap().is_empty() && bell.stored.lock().unwrap().is_empty());
}

#[tokio::test]
async fn test_the_publisher_keeps_live_notices_off_the_stored_and_indexed_topic() {
    use application_core::traits::{EventService, LIVE_NOTIFICATIONS_TOPIC, NOTIFICATIONS_TOPIC};
    use application_core::EventError;
    use infrastructure::services::RedpandaNotificationPublisher;

    #[derive(Default)]
    struct Broker(Mutex<Vec<(String, Option<String>)>>);
    #[async_trait]
    impl EventService for Broker {
        async fn publish(&self, topic: &str, key: Option<&str>, _payload: &[u8]) -> Result<(), EventError> {
            self.0.lock().unwrap().push((topic.to_string(), key.map(Into::into)));
            Ok(())
        }
    }

    let broker = Arc::new(Broker::default());
    let publisher = RedpandaNotificationPublisher::new(broker.clone());
    let n = || Notification {
        id: uuid::Uuid::now_v7().to_string(),
        kind: "data_updated".into(),
        title: "t".into(),
        body: "b".into(),
        data: serde_json::json!({}),
        link: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    publisher.notify("somchai", n()).await;
    publisher.notify_live("somchai", n()).await;
    let sent = broker.0.lock().unwrap().clone();
    assert_eq!(sent[0], (NOTIFICATIONS_TOPIC.to_string(), Some("somchai".to_string())));
    assert_eq!(sent[1], (LIVE_NOTIFICATIONS_TOPIC.to_string(), Some("somchai".to_string())), "its own topic (not indexed)");
}
