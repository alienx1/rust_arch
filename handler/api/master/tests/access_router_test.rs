//! Integration tests for `GET /access/me`: the signed-in account's rights per program.
//! Located at `handler/api/master/tests/access_router_test.rs`.

use api_master::routers::{ACCESS_PATH, AccessState, access_routes_with_state};
use application_master::features::su::access_control::{
    AccessControl, AccessError, Actor, OwnBranch, OwnEmployee, OwnUnit, PermissionAction,
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

    async fn own_employee(&self, username: &str) -> Result<Option<OwnEmployee>, AccessError> {
        Ok((username == "writer").then(|| OwnEmployee {
            id: uuid::Uuid::nil(),
            image_url: Some("/api/v1/db/dbpg003/employees/x/image?v=abc".to_string()),
            organization: Some(OwnUnit {
                id: uuid::Uuid::nil(),
                code: "ARCH".to_string(),
                name_th: "องค์กร ARCH".to_string(),
                name_en: "ARCH Organization".to_string(),
                image_url: Some("/api/v1/db/dbpg001/organizations/o/image?v=abc".to_string()),
            }),
            company: Some(OwnUnit {
                id: uuid::Uuid::nil(),
                code: "ARCH-HQ".to_string(),
                name_th: "บริษัท ARCH".to_string(),
                name_en: "ARCH Co.".to_string(),
                image_url: None,
            }),
            is_hq: true,
            unlimited: true,
            branches: vec![OwnBranch {
                id: uuid::Uuid::nil(),
                company_id: uuid::Uuid::nil(),
                code: "HQ".to_string(),
                name_th: "สำนักงานใหญ่".to_string(),
                name_en: "Head Office".to_string(),
            }],
            choices: vec![OwnUnit {
                id: uuid::Uuid::nil(),
                code: "ARCH-HQ".to_string(),
                name_th: "บริษัท ARCH".to_string(),
                name_en: "ARCH Co.".to_string(),
                image_url: None,
            }],
        }))
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

#[tokio::test]
async fn test_me_carries_the_photo_of_the_accounts_employee() {
    let (_, writer) = me("writer").await;
    assert_eq!(writer["image_url"], "/api/v1/db/dbpg003/employees/x/image?v=abc");
    let (_, stranger) = me("stranger").await;
    assert!(stranger["image_url"].is_null(), "no photo, no url");
}

#[tokio::test]
async fn test_me_carries_the_organization_and_company_for_the_sidebar_brand() {
    let (_, writer) = me("writer").await;
    assert_eq!(writer["organization"]["name_en"], "ARCH Organization");
    assert_eq!(writer["organization"]["image_url"], "/api/v1/db/dbpg001/organizations/o/image?v=abc");
    assert_eq!(writer["company"]["name_th"], "บริษัท ARCH");
    assert!(writer["company"]["image_url"].is_null());
    let (_, stranger) = me("stranger").await;
    assert!(stranger["organization"].is_null() && stranger["company"].is_null());
}

#[tokio::test]
async fn test_me_tells_headquarters_and_lists_the_companies_to_pick_from() {
    let (_, writer) = me("writer").await;
    assert_eq!(writer["is_hq"], true);
    assert_eq!(writer["companies"][0]["name_en"], "ARCH Co.");
    assert!(writer["selected_company_id"].is_null(), "no view scope here: the overview");
    assert_eq!(writer["branches"][0]["code"], "HQ");
    assert!(writer["selected_branch_id"].is_null());
    let (_, stranger) = me("stranger").await;
    assert_eq!(stranger["is_hq"], false);
    assert_eq!(stranger["companies"], serde_json::json!([]));
    assert_eq!(stranger["branches"], serde_json::json!([]));
}
