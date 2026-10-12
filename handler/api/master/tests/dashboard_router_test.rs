//! Integration tests for Dashboard router endpoints.

use api_master::routers::{DashboardState, dashboard_routes_with_state};
use application_master::features::su::access_control::{
    AccessControl, AccessError, Actor, PermissionAction,
};
use application_master::features::dashboard::{
    DashboardCommands, DashboardError, DashboardKpis, DashboardSummary, ModuleItem, ProgramItem,
};
use async_trait::async_trait;
use axum::{Extension, Router};
use infrastructure::middleware::AuthenticatedUser;
use uuid::Uuid;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

/// `admin` and `reader` may read the dashboard (program SYPG000); anyone else has nothing.
struct MockAccess;

#[async_trait]
impl AccessControl for MockAccess {
    async fn authorize(
        &self,
        username: &str,
        program_code: &str,
        action: PermissionAction,
    ) -> Result<Option<Actor>, AccessError> {
        assert_eq!(program_code, "SYPG000");
        let allowed = matches!(username, "admin" | "reader") && action == PermissionAction::Read;
        Ok(allowed.then(|| Actor { user_id: Uuid::nil(), username: username.to_string() }))
    }
}

fn user(username: &str) -> AuthenticatedUser {
    AuthenticatedUser {
        identity_id: "i".to_string(),
        session_id: "s".to_string(),
        email: Some(format!("{username}@arch.local")),
        traits: serde_json::json!({}),
    }
}

struct MockDashboardCommands {
    should_fail: bool,
}

#[async_trait]
impl DashboardCommands for MockDashboardCommands {
    async fn get_dashboard_summary(&self) -> Result<DashboardSummary, DashboardError> {
        if self.should_fail {
            return Err(DashboardError::Database(sea_orm::DbErr::Custom(
                "DB timeout".to_string(),
            )));
        }

        Ok(DashboardSummary {
            kpis: DashboardKpis {
                companies: 1,
                branches: 1,
                organizations: 1,
                departments: 1,
                employees: 2,
                provinces: 77,
                districts: 1008,
                sub_districts: 8939,
                labels: 292,
                languages: 2,
                systems: 4,
                modules: 4,
                programs: 21,
                users: 2,
            },
            modules: vec![
                ModuleItem {
                    code: "DBMT".to_string(),
                    label_th: "ทะเบียนข้อมูลหลัก".to_string(),
                    label_en: "Database Master Registration".to_string(),
                    system_code: "DB".to_string(),
                    active: true,
                },
                ModuleItem {
                    code: "SYMT".to_string(),
                    label_th: "ระบบงานหลัก".to_string(),
                    label_en: "System Core".to_string(),
                    system_code: "SY".to_string(),
                    active: true,
                },
            ],
            recent_programs: vec![ProgramItem {
                code: "DBPG001".to_string(),
                label_th: "จัดการบริษัท".to_string(),
                label_en: "Manage".to_string(),
                active: true,
            }],
            region: "ap-southeast-1 (Bangkok / Multi-Region)".to_string(),
            architecture: "Active-Active Spock pgEdge".to_string(),
        })
    }
}

fn app_as(should_fail: bool, username: &str) -> Router {
    let state = DashboardState {
        commands: Arc::new(MockDashboardCommands { should_fail }),
        access: Arc::new(MockAccess),
    };
    dashboard_routes_with_state(state).layer(Extension(user(username)))
}

fn test_app(should_fail: bool) -> Router {
    app_as(should_fail, "admin")
}

#[tokio::test]
async fn test_dashboard_requires_read_on_sypg000() {
    let get = || Request::builder().uri("/summary").body(Body::empty()).unwrap();

    let denied = app_as(false, "stranger").oneshot(get()).await.unwrap();
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    let bytes = to_bytes(denied.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["code"], "FORBIDDEN");

    let ok = app_as(false, "reader").oneshot(get()).await.unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_dashboard_summary_success() {
    let app = test_app(false);

    let req = Request::builder()
        .uri("/summary")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: DashboardSummary = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(summary.kpis.companies, 1);
    assert_eq!(summary.kpis.employees, 2);
    assert_eq!(summary.kpis.provinces, 77);
    assert_eq!(summary.kpis.labels, 292);
    assert_eq!(summary.modules.len(), 2);
    assert_eq!(summary.modules[0].code, "DBMT");
    assert_eq!(summary.recent_programs.len(), 1);
    assert_eq!(summary.architecture, "Active-Active Spock pgEdge");
}

#[tokio::test]
async fn test_get_dashboard_summary_cbor_content_negotiation() {
    let app = test_app(false);

    let req = Request::builder()
        .uri("/summary")
        .method("GET")
        .header("Accept", "application/cbor")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "application/cbor"
    );

    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: DashboardSummary = infrastructure::wire::from_cbor_slice(&bytes).unwrap();
    assert_eq!(summary.kpis.companies, 1);
    assert_eq!(summary.kpis.employees, 2);
    assert_eq!(summary.kpis.provinces, 77);
    assert_eq!(summary.modules.len(), 2);
    assert_eq!(summary.modules[0].code, "DBMT");
    assert_eq!(summary.architecture, "Active-Active Spock pgEdge");
}

#[tokio::test]
async fn test_get_dashboard_summary_database_error() {
    let app = test_app(true);

    let req = Request::builder()
        .uri("/summary")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let err: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(err["success"], false);
    assert_eq!(err["code"], "DATABASE_ERROR");
}
