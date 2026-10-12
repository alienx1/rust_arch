//! Integration tests for the ALPG000 router (`/al/alpg000/...`) and the audit events intake
//! (`/audit/events`). Located at `handler/api/master/tests/alpg000_router_test.rs`.

use api_master::routers::{
    ALPG000_PATH, AUDIT_EVENTS_PATH, Alpg000State, AuditEventsState, alpg000_routes_with_state,
    audit_events_routes_with_state,
};
use application_master::features::al::alpg000::{
    Alpg000Commands, Alpg000Error, AuditLogDto, ExportRequestedDto, AuditLogFilterQuery, AuditLogPageDto,
    AuditLogSummaryDto, AuditStatusDto, ClientActor, ClientEventCommand, ProgramOptionDto,
};
use application_master::features::su::access_control::{
    AccessControl, AccessError, Actor, PermissionAction,
};
use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum::{Extension, Router};
use infrastructure::middleware::AuthenticatedUser;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;
use uuid::Uuid;

/// `reader` may READ ALPG000; everybody else is refused
struct MockAccess;

#[async_trait]
impl AccessControl for MockAccess {
    async fn authorize(
        &self,
        username: &str,
        program_code: &str,
        action: PermissionAction,
    ) -> Result<Option<Actor>, AccessError> {
        assert_eq!(program_code, "ALPG000");
        let allowed = username == "reader" && matches!(action, PermissionAction::Read);
        Ok(allowed.then(|| Actor { user_id: Uuid::nil(), username: username.to_string() }))
    }
}

#[derive(Default)]
struct MockCommands {
    events: Mutex<Vec<(ClientActor, Vec<ClientEventCommand>)>>,
}

#[async_trait]
impl Alpg000Commands for MockCommands {
    async fn search_logs(&self, f: AuditLogFilterQuery) -> Result<AuditLogPageDto, Alpg000Error> {
        if f.text.as_deref() == Some("down") {
            return Err(Alpg000Error::Unavailable("quickwit".into()));
        }
        if f.text.as_deref() == Some("bad") {
            return Err(Alpg000Error::Validation("bad".into()));
        }
        Ok(AuditLogPageDto {
            items: vec![AuditLogSummaryDto {
                event_id: "e1".into(),
                kind: f.kind.unwrap_or_default(),
                username: f.user,
                status: f.status,
                ..AuditLogSummaryDto::default()
            }],
            total: 7,
            page: f.page.unwrap_or(1),
            limit: f.limit.unwrap_or(25),
        })
    }
    async fn request_export(&self, by: &str, f: AuditLogFilterQuery) -> Result<ExportRequestedDto, Alpg000Error> {
        if f.text.as_deref() == Some("down") {
            return Err(Alpg000Error::Unavailable("redpanda".into()));
        }
        Ok(ExportRequestedDto { job_id: format!("job-for-{by}") })
    }
    async fn get_log(&self, id: &str) -> Result<Option<AuditLogDto>, Alpg000Error> {
        Ok((id == "e1").then(|| AuditLogDto { event_id: "e1".into(), ..AuditLogDto::default() }))
    }
    async fn program_options(&self) -> Result<Vec<ProgramOptionDto>, Alpg000Error> {
        Ok(vec![ProgramOptionDto {
            code: "DBPG004".to_string(),
            name_th: "ข้อมูลอ้างอิง".to_string(),
            name_en: "Reference Data".to_string(),
        }])
    }
    async fn status(&self) -> AuditStatusDto {
        AuditStatusDto { available: true }
    }
    async fn record_client_events(
        &self,
        actor: ClientActor,
        events: Vec<ClientEventCommand>,
    ) -> Result<usize, Alpg000Error> {
        if events.len() > 50 {
            return Err(Alpg000Error::Validation("too many".into()));
        }
        let n = events.len();
        self.events.lock().unwrap().push((actor, events));
        Ok(n)
    }
}

fn user(username: &str) -> AuthenticatedUser {
    AuthenticatedUser {
        identity_id: "id-1".to_string(),
        session_id: "sess-1".to_string(),
        email: Some(format!("{username}@arch.local")),
        traits: serde_json::json!({}),
    }
}

fn logs_router(username: &str) -> Router {
    let state = Alpg000State { commands: Arc::new(MockCommands::default()), access: Arc::new(MockAccess) };
    Router::new().nest(ALPG000_PATH, alpg000_routes_with_state(state)).layer(Extension(user(username)))
}

fn events_router(commands: Arc<MockCommands>) -> Router {
    Router::new()
        .nest(AUDIT_EVENTS_PATH, audit_events_routes_with_state(AuditEventsState { commands }))
        .layer(Extension(user("somchai")))
}

fn get(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).method("GET").body(Body::empty()).unwrap()
}

async fn status_and_json(resp: axum::response::Response) -> (StatusCode, serde_json::Value) {
    let status = resp.status();
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null))
}

fn post_events(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .uri("/audit/events")
        .method("POST")
        .header("content-type", "application/json")
        .header("x-forwarded-for", "203.0.113.5, 10.0.0.1")
        .header("user-agent", "TestAgent/1.0")
        .header("accept-language", "th-TH,th;q=0.9,en;q=0.8")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn test_every_endpoint_needs_read() {
    for uri in ["/al/alpg000/logs", "/al/alpg000/logs/e1", "/al/alpg000/status"] {
        assert_eq!(logs_router("stranger").oneshot(get(uri)).await.unwrap().status(), StatusCode::FORBIDDEN, "{uri}");
    }
}

#[tokio::test]
async fn test_list_parses_the_query() {
    let (status, body) = status_and_json(
        logs_router("reader")
            .oneshot(get("/al/alpg000/logs?kind=api&user=somchai&status=404&page=2&limit=10"))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"][0]["kind"], "api");
    assert_eq!(body["items"][0]["username"], "somchai");
    assert_eq!(body["items"][0]["status"], 404);
    assert_eq!((body["total"].as_u64(), body["page"].as_u64(), body["limit"].as_u64()), (Some(7), Some(2), Some(10)));
}

#[tokio::test]
async fn test_detail_and_status() {
    let (status, body) = status_and_json(logs_router("reader").oneshot(get("/al/alpg000/logs/e1")).await.unwrap()).await;
    assert_eq!((status, body["event_id"].as_str()), (StatusCode::OK, Some("e1")));

    let (status, body) = status_and_json(logs_router("reader").oneshot(get("/al/alpg000/logs/zzz")).await.unwrap()).await;
    assert_eq!((status, body["code"].as_str()), (StatusCode::NOT_FOUND, Some("NOT_FOUND")));

    let (status, body) = status_and_json(logs_router("reader").oneshot(get("/al/alpg000/status")).await.unwrap()).await;
    assert_eq!((status, body["available"].as_bool()), (StatusCode::OK, Some(true)));
}

#[tokio::test]
async fn test_store_down_is_503_and_bad_filter_400() {
    let (status, body) = status_and_json(logs_router("reader").oneshot(get("/al/alpg000/logs?text=down")).await.unwrap()).await;
    assert_eq!((status, body["code"].as_str()), (StatusCode::SERVICE_UNAVAILABLE, Some("AUDIT_UNAVAILABLE")));
    let (status, body) = status_and_json(logs_router("reader").oneshot(get("/al/alpg000/logs?text=bad")).await.unwrap()).await;
    assert_eq!((status, body["code"].as_str()), (StatusCode::BAD_REQUEST, Some("VALIDATION_ERROR")));
}

#[tokio::test]
async fn test_events_are_attributed_to_the_caller() {
    let commands = Arc::new(MockCommands::default());
    let resp = events_router(commands.clone())
        .oneshot(post_events(serde_json::json!({
            "events": [{ "kind": "click", "page": "/db/dbpg004", "element": "button#save", "label": "Save" }]
        })))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let seen = commands.events.lock().unwrap();
    let (actor, events) = &seen[0];
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].element.as_deref(), Some("button#save"));
    assert_eq!(actor.user_id.as_deref(), Some("id-1"));
    assert_eq!(actor.session_id.as_deref(), Some("sess-1"));
    assert_eq!(actor.username.as_deref(), Some("somchai"));
    assert_eq!(actor.email.as_deref(), Some("somchai@arch.local"));
    assert_eq!(actor.ip.as_deref(), Some("203.0.113.5"));
    assert_eq!(actor.user_agent.as_deref(), Some("TestAgent/1.0"));
    assert_eq!(actor.language.as_deref(), Some("th"));
}

#[tokio::test]
async fn test_oversized_or_malformed_events_are_400() {
    let events: Vec<_> = (0..51).map(|_| serde_json::json!({ "kind": "click" })).collect();
    let resp = events_router(Arc::new(MockCommands::default()))
        .oneshot(post_events(serde_json::json!({ "events": events })))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let resp = events_router(Arc::new(MockCommands::default()))
        .oneshot(post_events(serde_json::json!({ "events": "nope" })))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_program_options_need_read_and_list_the_catalogue() {
    let (status, body) =
        status_and_json(logs_router("reader").oneshot(get("/al/alpg000/programs")).await.unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body[0]["code"], "DBPG004");
    assert_eq!(body[0]["name_en"], "Reference Data");

    let denied = logs_router("stranger").oneshot(get("/al/alpg000/programs")).await.unwrap();
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_export_request_is_accepted_with_the_job_id_of_the_requester() {
    let req = Request::builder().uri("/al/alpg000/exports?kind=api").method("POST").body(Body::empty()).unwrap();
    let (status, json) = status_and_json(logs_router("reader").oneshot(req).await.unwrap()).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(json["job_id"], "job-for-reader");
}

#[tokio::test]
async fn test_export_request_needs_read_and_reports_a_down_queue_as_503() {
    let post = |uri: &str| Request::builder().uri(uri).method("POST").body(Body::empty()).unwrap();
    assert_eq!(
        logs_router("stranger").oneshot(post("/al/alpg000/exports")).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
    let (status, json) =
        status_and_json(logs_router("reader").oneshot(post("/al/alpg000/exports?text=down")).await.unwrap()).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(json["code"], "AUDIT_UNAVAILABLE");
}

#[tokio::test]
async fn test_exports_cannot_be_fetched_with_get() {
    assert_eq!(
        logs_router("reader").oneshot(get("/al/alpg000/exports")).await.unwrap().status(),
        StatusCode::METHOD_NOT_ALLOWED
    );
}
