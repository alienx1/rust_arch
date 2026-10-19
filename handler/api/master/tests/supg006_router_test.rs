//! Integration tests for SUPG006 router endpoints (`/su/supg006/config`) including authorization.
//! Located at `handler/api/master/tests/supg006_router_test.rs`.

use api_master::routers::{SUPG006_PATH, Supg006State, supg006_routes_with_state};
use application_master::features::su::access_control::{AccessControl, AccessError, Actor, PermissionAction};
use application_master::features::su::supg006::{
    ConfigGroup, ConfigItemDto, ConfigKind, SaveConfigCommand, Supg006Commands, Supg006Error, validate_change,
};
use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum::{Extension, Router};
use infrastructure::middleware::AuthenticatedUser;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

/// `writer` may read and write, `reader` may only read.
struct MockAccess;

#[async_trait]
impl AccessControl for MockAccess {
    async fn authorize(&self, username: &str, program_code: &str, action: PermissionAction) -> Result<Option<Actor>, AccessError> {
        assert_eq!(program_code, "SUPG006");
        let allowed = match (username, action) {
            ("writer", _) => true,
            ("reader", PermissionAction::Read) => true,
            _ => false,
        };
        Ok(allowed.then(|| Actor { user_id: Uuid::nil(), username: username.to_string() }))
    }
}

fn item(key: &str, value: &str, row_version: i64) -> ConfigItemDto {
    ConfigItemDto {
        key: key.to_string(),
        group: ConfigGroup::Login,
        kind: ConfigKind::Number,
        value: value.to_string(),
        default_value: "5".to_string(),
        min: Some(0),
        max: Some(100),
        description: None,
        row_version,
    }
}

struct MockCommands;

#[async_trait]
impl Supg006Commands for MockCommands {
    async fn list_config(&self) -> Result<Vec<ConfigItemDto>, Supg006Error> {
        Ok(vec![item("security.login.max_failed_attempts", "5", 2)])
    }

    async fn save_config(&self, cmd: SaveConfigCommand, _actor: Option<Uuid>) -> Result<Vec<ConfigItemDto>, Supg006Error> {
        let mut out = Vec::new();
        for c in &cmd.changes {
            let value = validate_change(c)?;
            // The mock treats version 99 as a copy somebody else saved over
            if c.row_version == 99 {
                return Err(Supg006Error::VersionConflict);
            }
            out.push(item(&c.key, &value, c.row_version + 1));
        }
        Ok(out)
    }
}

fn router(username: &str) -> Router {
    let state = Supg006State { commands: Arc::new(MockCommands), access: Arc::new(MockAccess) };
    Router::new().nest(SUPG006_PATH, supg006_routes_with_state(state)).layer(Extension(AuthenticatedUser {
        identity_id: "i".to_string(),
        session_id: "s".to_string(),
        email: Some(format!("{username}@arch.local")),
        traits: serde_json::json!({}),
    }))
}

fn put(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .uri("/su/supg006/config")
        .method("PUT")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

async fn status_and_json(resp: axum::response::Response) -> (StatusCode, serde_json::Value) {
    let status = resp.status();
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null))
}

#[tokio::test]
async fn test_reading_needs_read_and_saving_needs_write() {
    let get = || Request::builder().uri("/su/supg006/config").method("GET").body(Body::empty()).unwrap();
    assert_eq!(router("stranger").oneshot(get()).await.unwrap().status(), StatusCode::FORBIDDEN);
    let (status, body) = status_and_json(router("reader").oneshot(get()).await.unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body[0]["key"], "security.login.max_failed_attempts");
    assert_eq!(body[0]["row_version"], 2);

    let change = serde_json::json!({ "changes": [{ "key": "security.login.lock_minutes", "value": "30", "row_version": 1 }] });
    assert_eq!(router("reader").oneshot(put(change.clone())).await.unwrap().status(), StatusCode::FORBIDDEN);
    let (status, body) = status_and_json(router("writer").oneshot(put(change)).await.unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!((body[0]["value"].as_str(), body[0]["row_version"].as_i64()), (Some("30"), Some(2)));
}

#[tokio::test]
async fn test_a_wrong_value_is_400_and_a_stale_copy_is_412() {
    let bad = serde_json::json!({ "changes": [{ "key": "security.login.lock_minutes", "value": "0", "row_version": 1 }] });
    let (status, body) = status_and_json(router("writer").oneshot(put(bad)).await.unwrap()).await;
    assert_eq!((status, body["code"].as_str()), (StatusCode::BAD_REQUEST, Some("VALIDATION_ERROR")));

    let stale = serde_json::json!({ "changes": [{ "key": "security.login.lock_minutes", "value": "30", "row_version": 99 }] });
    let (status, body) = status_and_json(router("writer").oneshot(put(stale)).await.unwrap()).await;
    assert_eq!((status, body["code"].as_str()), (StatusCode::PRECONDITION_FAILED, Some("VERSION_CONFLICT")));

    let outside = serde_json::json!({ "changes": [{ "key": "mail.smtp_url", "value": "smtp://x", "row_version": 1 }] });
    assert_eq!(router("writer").oneshot(put(outside)).await.unwrap().status(), StatusCode::BAD_REQUEST);
}
