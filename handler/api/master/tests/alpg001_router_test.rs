//! Integration tests for the ALPG001 router (`/al/alpg001/...`). Located at
//! `handler/api/master/tests/alpg001_router_test.rs`.

use api_master::routers::{ALPG001_PATH, Alpg001State, alpg001_routes_with_state};
use application_master::features::al::alpg001::{
    Alpg001Commands, Alpg001Error, ProcessCountsDto, ProcessDto, ProcessFilterQuery, ProcessPageDto,
    ReprocessRequestedDto,
};
use application_master::features::su::access_control::{AccessControl, AccessError, Actor, PermissionAction};
use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum::{Extension, Router};
use infrastructure::middleware::AuthenticatedUser;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

/// `reader` may READ ALPG001, `operator` may READ and WRITE it; everybody else is refused
struct MockAccess;

#[async_trait]
impl AccessControl for MockAccess {
    async fn authorize(
        &self,
        username: &str,
        program_code: &str,
        action: PermissionAction,
    ) -> Result<Option<Actor>, AccessError> {
        assert_eq!(program_code, "ALPG001");
        let allowed = match action {
            PermissionAction::Read => matches!(username, "reader" | "operator"),
            PermissionAction::Write => username == "operator",
            PermissionAction::Approve => false,
        };
        Ok(allowed.then(|| Actor { user_id: Uuid::nil(), username: username.to_string() }))
    }
}

fn process(id: &str, status: &str) -> ProcessDto {
    ProcessDto {
        id: id.into(),
        kind: "audit_log_csv".into(),
        requested_by: "somchai".into(),
        status: status.into(),
        row_count: 0,
        size_bytes: 0,
        file_name: None,
        error: None,
        requested_at: "2026-10-12T00:00:00Z".into(),
        started_at: None,
        finished_at: None,
        duration_ms: None,
        stuck: false,
        can_reprocess: status == "failed",
        filter: serde_json::json!({}),
    }
}

struct MockCommands;

#[async_trait]
impl Alpg001Commands for MockCommands {
    async fn list_processes(&self, f: ProcessFilterQuery) -> Result<ProcessPageDto, Alpg001Error> {
        if f.user.as_deref() == Some("bad") {
            return Err(Alpg001Error::Validation("bad".into()));
        }
        Ok(ProcessPageDto {
            items: vec![process("p1", f.status.as_deref().unwrap_or("failed"))],
            total: 1,
            page: f.page.unwrap_or(1),
            limit: f.limit.unwrap_or(25),
            counts: ProcessCountsDto { failed: 1, ..Default::default() },
        })
    }
    async fn get_process(&self, id: &str) -> Result<Option<ProcessDto>, Alpg001Error> {
        Ok((id == "p1").then(|| process("p1", "failed")))
    }
    async fn reprocess(&self, id: &str) -> Result<ReprocessRequestedDto, Alpg001Error> {
        match id {
            "p1" => Ok(ReprocessRequestedDto { id: id.into() }),
            "running" => Err(Alpg001Error::Conflict("running".into())),
            "down" => Err(Alpg001Error::Unavailable("redpanda".into())),
            _ => Err(Alpg001Error::NotFound(id.into())),
        }
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

fn app(username: &str) -> Router {
    let state = Alpg001State { commands: Arc::new(MockCommands), access: Arc::new(MockAccess) };
    Router::new().nest(ALPG001_PATH, alpg001_routes_with_state(state)).layer(Extension(user(username)))
}

fn request(method: &str, uri: &str) -> Request<Body> {
    Request::builder().uri(uri).method(method).body(Body::empty()).unwrap()
}

async fn status_and_json(resp: axum::response::Response) -> (StatusCode, serde_json::Value) {
    let status = resp.status();
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null))
}

#[tokio::test]
async fn test_list_returns_the_page_with_counters_for_a_reader() {
    let resp = app("reader").oneshot(request("GET", "/al/alpg001/processes?status=failed&limit=10")).await.unwrap();
    let (status, json) = status_and_json(resp).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["total"], 1);
    assert_eq!(json["limit"], 10);
    assert_eq!(json["items"][0]["status"], "failed");
    assert_eq!(json["items"][0]["can_reprocess"], true);
    assert_eq!(json["counts"]["failed"], 1);
}

#[tokio::test]
async fn test_list_without_read_is_forbidden_and_a_bad_filter_is_400() {
    let (status, _) = status_and_json(app("stranger").oneshot(request("GET", "/al/alpg001/processes")).await.unwrap()).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, json) =
        status_and_json(app("reader").oneshot(request("GET", "/al/alpg001/processes?user=bad")).await.unwrap()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_get_one_process_or_404() {
    let (status, json) = status_and_json(app("reader").oneshot(request("GET", "/al/alpg001/processes/p1")).await.unwrap()).await;
    assert_eq!((status, json["id"].as_str()), (StatusCode::OK, Some("p1")));
    let (status, json) = status_and_json(app("reader").oneshot(request("GET", "/al/alpg001/processes/zz")).await.unwrap()).await;
    assert_eq!((status, json["code"].as_str()), (StatusCode::NOT_FOUND, Some("NOT_FOUND")));
}

#[tokio::test]
async fn test_reprocess_needs_write_and_answers_202() {
    let (status, _) =
        status_and_json(app("reader").oneshot(request("POST", "/al/alpg001/processes/p1/reprocess")).await.unwrap()).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "READ alone cannot reprocess");
    let (status, json) =
        status_and_json(app("operator").oneshot(request("POST", "/al/alpg001/processes/p1/reprocess")).await.unwrap()).await;
    assert_eq!((status, json["id"].as_str()), (StatusCode::ACCEPTED, Some("p1")));
}

#[tokio::test]
async fn test_reprocess_errors_map_to_409_503_404() {
    for (id, expected, code) in [
        ("running", StatusCode::CONFLICT, "PROCESS_NOT_REPROCESSABLE"),
        ("down", StatusCode::SERVICE_UNAVAILABLE, "PROCESS_QUEUE_UNAVAILABLE"),
        ("zz", StatusCode::NOT_FOUND, "NOT_FOUND"),
    ] {
        let uri = format!("/al/alpg001/processes/{id}/reprocess");
        let (status, json) = status_and_json(app("operator").oneshot(request("POST", &uri)).await.unwrap()).await;
        assert_eq!((status, json["code"].as_str()), (expected, Some(code)), "for {id}");
    }
}
