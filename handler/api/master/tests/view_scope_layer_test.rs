//! The view-scope middleware: who is limited to what, and that it never fails open.
//! Located at `handler/api/master/tests/view_scope_layer_test.rs`.

use api_master::view_scope_layer::{BRANCH_HEADER, COMPANY_HEADER, apply_view_scope};
use application_master::features::su::access_control::{
    AccessControl, AccessError, Actor, GroupReach, PermissionAction, ReachedBranch, Viewer,
};
use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum::{Extension, Router, routing::get};
use infrastructure::middleware::AuthenticatedUser;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

const ORG: Uuid = Uuid::from_u128(1);
const HQ_COMPANY: Uuid = Uuid::from_u128(10);
const OTHER_COMPANY: Uuid = Uuid::from_u128(11);
const FOREIGN_COMPANY: Uuid = Uuid::from_u128(99);
const BRANCH: Uuid = Uuid::from_u128(20);
/// A branch of `OTHER_COMPANY`
const OTHER_BRANCH: Uuid = Uuid::from_u128(21);
const FOREIGN_BRANCH: Uuid = Uuid::from_u128(98);
const PLAIN_GROUP: Uuid = Uuid::from_u128(30);
const DASH_GROUP: Uuid = Uuid::from_u128(31);

struct MockAccess;

#[async_trait]
impl AccessControl for MockAccess {
    async fn authorize(&self, _u: &str, _p: &str, _a: PermissionAction) -> Result<Option<Actor>, AccessError> {
        Ok(None)
    }

    async fn viewer(&self, username: &str) -> Result<Option<Viewer>, AccessError> {
        match username {
            "hq" => Ok(Some(Viewer {
                is_hq: true,
                unlimited: true,
                branches: vec![],
                departments: vec![],
                ..Viewer::own_branch(ORG, HQ_COMPANY, BRANCH)
            })),
            "branch" => Ok(Some(Viewer::own_branch(ORG, OTHER_COMPANY, BRANCH))),
            // Registered in BRANCH; one group reaches BRANCH, another (the dashboard's) OTHER_BRANCH
            "dash" => Ok(Some(Viewer {
                branches: vec![
                    ReachedBranch { branch_id: BRANCH, company_id: OTHER_COMPANY, whole: true },
                    ReachedBranch { branch_id: OTHER_BRANCH, company_id: OTHER_COMPANY, whole: true },
                ],
                reaches: vec![
                    GroupReach { group_id: PLAIN_GROUP, unlimited: false, departments: vec![], branches: vec![ReachedBranch { branch_id: BRANCH, company_id: OTHER_COMPANY, whole: true }] },
                    GroupReach { group_id: DASH_GROUP, unlimited: false, departments: vec![], branches: vec![ReachedBranch { branch_id: OTHER_BRANCH, company_id: OTHER_COMPANY, whole: true }] },
                ],
                ..Viewer::own_branch(ORG, OTHER_COMPANY, BRANCH)
            })),
            // Works in BRANCH of OTHER_COMPANY and its group reaches OTHER_BRANCH (whole) as well
            "multi" => Ok(Some(Viewer {
                branches: vec![
                    ReachedBranch { branch_id: BRANCH, company_id: OTHER_COMPANY, whole: true },
                    ReachedBranch { branch_id: OTHER_BRANCH, company_id: OTHER_COMPANY, whole: true },
                ],
                ..Viewer::own_branch(ORG, OTHER_COMPANY, BRANCH)
            })),
            "broken" => Err(AccessError::Database("down".to_string())),
            _ => Ok(None),
        }
    }

    async fn granting_groups(&self, username: &str, program_code: &str, _a: PermissionAction) -> Result<Option<Vec<Uuid>>, AccessError> {
        // `dash` holds the dashboard in OTHER_BRANCH only, and the plain group in the home branch
        Ok(match (username, program_code) {
            ("dash", "SYPG000") => Some(vec![DASH_GROUP]),
            ("dash", _) => Some(vec![PLAIN_GROUP]),
            _ => None,
        })
    }

    async fn branch_company(&self, organization_id: Uuid, branch_id: Uuid) -> Result<Option<Uuid>, AccessError> {
        Ok(match (organization_id == ORG, branch_id) {
            (true, OTHER_BRANCH) => Some(OTHER_COMPANY),
            (true, BRANCH) => Some(HQ_COMPANY),
            _ => None,
        })
    }

    async fn company_in_organization(&self, organization_id: Uuid, company_id: Uuid) -> Result<bool, AccessError> {
        Ok(organization_id == ORG && company_id != FOREIGN_COMPANY)
    }
}

/// Answers with the scope the handler sees
async fn scope_seen() -> axum::Json<serde_json::Value> {
    let scope = application_core::view_scope::current();
    axum::Json(serde_json::json!({
        "company": scope.as_ref().and_then(|s| s.single_company()),
        "branch": scope.as_ref().and_then(|s| s.single_branch()),
        "branches": scope.as_ref().and_then(|s| s.branches.clone()),
    }))
}

async fn call(username: &str, header: Option<Uuid>) -> (StatusCode, serde_json::Value) {
    call_with(username, header, None).await
}

async fn call_with(username: &str, header: Option<Uuid>, branch: Option<Uuid>) -> (StatusCode, serde_json::Value) {
    call_at("/x", username, header, branch).await
}

async fn call_at(path: &str, username: &str, header: Option<Uuid>, branch: Option<Uuid>) -> (StatusCode, serde_json::Value) {
    let app = Router::new()
        .route("/x", get(scope_seen))
        .route("/dashboard/summary", get(scope_seen))
        .layer(axum::middleware::from_fn_with_state(Arc::new(MockAccess) as Arc<dyn AccessControl>, apply_view_scope))
        .layer(Extension(AuthenticatedUser {
            identity_id: "i".to_string(),
            session_id: "s".to_string(),
            email: Some(format!("{username}@arch.local")),
            traits: serde_json::json!({}),
        }));
    let mut req = Request::builder().uri(path);
    if let Some(id) = header {
        req = req.header(COMPANY_HEADER, id.to_string());
    }
    if let Some(id) = branch {
        req = req.header(BRANCH_HEADER, id.to_string());
    }
    let resp = app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap();
    let status = resp.status();
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null))
}

#[tokio::test]
async fn test_headquarters_sees_everything_until_it_picks_a_company() {
    let (_, all) = call("hq", None).await;
    assert_eq!((all["company"].is_null(), all["branch"].is_null()), (true, true));

    let (_, one) = call("hq", Some(OTHER_COMPANY)).await;
    assert_eq!(one["company"], OTHER_COMPANY.to_string());
    assert!(one["branch"].is_null(), "headquarters is not limited to a branch");
}

#[tokio::test]
async fn test_headquarters_cannot_pick_a_company_of_another_organization() {
    let (_, seen) = call("hq", Some(FOREIGN_COMPANY)).await;
    assert!(seen["company"].is_null(), "falls back to the overview");
}

#[tokio::test]
async fn test_a_branch_account_sees_its_own_branch_and_cannot_widen_it() {
    for header in [None, Some(HQ_COMPANY), Some(FOREIGN_COMPANY)] {
        let (_, seen) = call("branch", header).await;
        assert_eq!(seen["company"], OTHER_COMPANY.to_string());
        assert_eq!(seen["branch"], BRANCH.to_string());
    }
}

#[tokio::test]
async fn test_an_account_that_cannot_be_placed_is_not_narrowed() {
    let (status, seen) = call("nobody", Some(HQ_COMPANY)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(seen["company"].is_null() && seen["branch"].is_null());
}

#[tokio::test]
async fn test_a_failed_lookup_is_an_error_never_an_unlimited_view() {
    let (status, _) = call("broken", None).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn test_headquarters_can_narrow_to_one_branch_whose_company_comes_along() {
    // Even when the picked company is another one, the branch decides
    let (_, seen) = call_with("hq", Some(HQ_COMPANY), Some(OTHER_BRANCH)).await;
    assert_eq!(seen["branch"], OTHER_BRANCH.to_string());
    assert_eq!(seen["company"], OTHER_COMPANY.to_string());
}

#[tokio::test]
async fn test_a_branch_outside_the_organization_is_ignored_for_headquarters() {
    let (_, seen) = call_with("hq", Some(OTHER_COMPANY), Some(FOREIGN_BRANCH)).await;
    assert!(seen["branch"].is_null(), "not a choice");
    assert_eq!(seen["company"], OTHER_COMPANY.to_string(), "the company pick still counts");
}

#[tokio::test]
async fn test_a_branch_account_cannot_pick_another_branch() {
    let (_, seen) = call_with("branch", None, Some(OTHER_BRANCH)).await;
    assert_eq!(seen["branch"], BRANCH.to_string());
    assert_eq!(seen["company"], OTHER_COMPANY.to_string());
}

#[tokio::test]
async fn test_an_account_whose_group_reaches_more_branches_sees_all_of_them() {
    let (_, seen) = call("multi", None).await;
    let mut branches: Vec<String> = serde_json::from_value(seen["branches"].clone()).unwrap();
    branches.sort();
    assert_eq!(branches, vec![BRANCH.to_string(), OTHER_BRANCH.to_string()]);
    assert!(seen["branch"].is_null(), "two branches: no single one");
    assert_eq!(seen["company"], OTHER_COMPANY.to_string());
}

#[tokio::test]
async fn test_such_an_account_can_narrow_to_a_branch_it_reaches_but_not_to_one_it_does_not() {
    let (_, narrowed) = call_with("multi", None, Some(OTHER_BRANCH)).await;
    assert_eq!(narrowed["branch"], OTHER_BRANCH.to_string());

    let (_, ignored) = call_with("multi", None, Some(FOREIGN_BRANCH)).await;
    assert_eq!(serde_json::from_value::<Vec<String>>(ignored["branches"].clone()).unwrap().len(), 2, "the pick is ignored: both branches stay");
}

#[tokio::test]
async fn test_the_dashboard_shows_only_the_branch_where_its_permission_is_held() {
    // Everywhere else both branches show; the dashboard is held in OTHER_BRANCH only
    let (_, other_screen) = call_at("/x", "dash", None, None).await;
    assert_eq!(serde_json::from_value::<Vec<String>>(other_screen["branches"].clone()).unwrap().len(), 2);
    let (status, dashboard) = call_at("/dashboard/summary", "dash", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(serde_json::from_value::<Vec<String>>(dashboard["branches"].clone()).unwrap(), vec![OTHER_BRANCH.to_string()]);
}

#[tokio::test]
async fn test_switching_to_a_branch_where_the_screen_is_not_granted_is_refused() {
    // The account's main branch has no dashboard: the dashboard is forbidden there ...
    let (status, body) = call_at("/dashboard/summary", "dash", None, Some(BRANCH)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "FORBIDDEN");
    // ... while the other screens of that branch still open, and the dashboard opens where it is granted
    assert_eq!(call_at("/x", "dash", None, Some(BRANCH)).await.0, StatusCode::OK);
    assert_eq!(call_at("/dashboard/summary", "dash", None, Some(OTHER_BRANCH)).await.0, StatusCode::OK);
}
