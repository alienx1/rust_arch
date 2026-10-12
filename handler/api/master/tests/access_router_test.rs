//! Integration tests for `GET /access/me`: the signed-in account's rights per program.
//! Located at `handler/api/master/tests/access_router_test.rs`.

use api_master::routers::{ACCESS_PATH, AccessState, access_routes_with_state};
use application_master::features::su::access_control::{
    AccessControl, AccessError, Actor, PermissionAction,
};
use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum::{Extension, Router};
use infrastructure::middleware::AuthenticatedUser;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use tower::ServiceExt;

struct MockAccess;

#[async_trait]
impl AccessControl for MockAccess {
    async fn authorize(&self, _u: &str, _p: &str, _a: PermissionAction) -> Result<Option<Actor>, AccessError> {
        Ok(None)
    }

    async fn permissions(
        &self,
        username: &str,
    ) -> Result<BTreeMap<String, BTreeSet<&'static str>>, AccessError> {
        let mut out = BTreeMap::new();
        if username == "writer" {
            out.insert("DBPG004".to_string(), BTreeSet::from(["READ", "WRITE"]));
            out.insert("SUPG001".to_string(), BTreeSet::from(["READ"]));
        }
        Ok(out)
    }
}

fn router(username: &str) -> Router {
    Router::new()
        .nest(ACCESS_PATH, access_routes_with_state(AccessState { access: Arc::new(MockAccess) }))
        .layer(Extension(AuthenticatedUser {
            identity_id: "i".to_string(),
            session_id: "s".to_string(),
            email: Some(format!("{username}@arch.local")),
            traits: serde_json::json!({}),
        }))
}

async fn me(username: &str) -> (StatusCode, serde_json::Value) {
    let resp = router(username)
        .oneshot(Request::builder().uri("/access/me").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null))
}

#[tokio::test]
async fn test_me_lists_programs_with_their_actions() {
    let (status, body) = me("writer").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["username"], "writer");
    assert_eq!(body["permissions"]["DBPG004"], serde_json::json!(["READ", "WRITE"]));
    assert_eq!(body["permissions"]["SUPG001"], serde_json::json!(["READ"]));
}

#[tokio::test]
async fn test_account_without_grants_gets_an_empty_map() {
    let (status, body) = me("stranger").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["permissions"], serde_json::json!({}));
}
