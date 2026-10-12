//! Integration tests for SUPG005 router endpoints (`/su/supg005/...`) and the Kratos hook.
//! Located at `handler/api/master/tests/supg005_router_test.rs`.

use api_master::routers::{SUPG005_PATH, Supg005State, supg005_routes_with_state};
use application_master::features::su::access_control::{
    AccessControl, AccessError, Actor, PermissionAction,
};
use application_master::features::su::supg005::{
    ActiveDeviceDto, Supg005Commands, Supg005Error, PaginatedResult, UserPaginationQuery,
    UserSessionOverviewDto,
};
use async_trait::async_trait;
use axum::{Extension, Router};
use infrastructure::middleware::AuthenticatedUser;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

struct MockSupg005Commands {
    should_fail: bool,
}

#[async_trait]
impl Supg005Commands for MockSupg005Commands {
    async fn list_users(&self) -> Result<Vec<UserSessionOverviewDto>, Supg005Error> {
        if self.should_fail {
            return Err(Supg005Error::Database("Mock DB error".to_string()));
        }

        Ok(vec![UserSessionOverviewDto {
            user_id: "019ca1ec-0000-7000-8010-000000000001".to_string(),
            username: "admin".to_string(),
            email: "admin@arch.local".to_string(),
            full_name: "ผู้ดูแล ระบบ".to_string(),
            employee_code: "EMP0001".to_string(),
            role_code: "ADMIN".to_string(),
            role_name: "ผู้ดูแลระบบสูงสุด".to_string(),
            scope_level: "organization".to_string(),
            department_name: "ฝ่ายเทคโนโลยีสารสนเทศ".to_string(),
            company_name: "สำนักงานใหญ่".to_string(),
            is_active_account: true,
            devices: vec![ActiveDeviceDto {
                id: "dev-01".to_string(),
                user_id: "019ca1ec-0000-7000-8010-000000000001".to_string(),
                device_name: "Chrome บน Linux".to_string(),
                ip_address: "127.0.0.1".to_string(),
                location: "กรุงเทพมหานคร".to_string(),
                last_active: "กำลังใช้งาน".to_string(),
                authenticated_at: "2026-10-01 08:30".to_string(),
                is_current: true,
            }],
        }])
    }

    async fn list_users_paginated(
        &self,
        query: UserPaginationQuery,
    ) -> Result<PaginatedResult<UserSessionOverviewDto>, Supg005Error> {
        if self.should_fail {
            return Err(Supg005Error::Database("Mock DB error".to_string()));
        }

        let users = self.list_users().await?;
        let total = users.len() as u64;
        let limit = query.limit.unwrap_or(5);
        let page = query.page.unwrap_or(1);
        let total_pages = if total == 0 { 1 } else { (total + limit - 1) / limit };

        Ok(PaginatedResult {
            items: users,
            total,
            page,
            limit,
            total_pages,
        })
    }

    async fn revoke_user_sessions(&self, _user_id: &str) -> Result<bool, Supg005Error> {
        if self.should_fail {
            return Err(Supg005Error::Internal("Failed to revoke user".to_string()));
        }
        Ok(true)
    }

    async fn suspend_user(&self, _user_id: &str) -> Result<bool, Supg005Error> {
        if self.should_fail {
            return Err(Supg005Error::Internal("Failed to suspend user".to_string()));
        }
        Ok(true)
    }

    async fn reactivate_user(&self, _user_id: &str) -> Result<bool, Supg005Error> {
        Ok(true)
    }

    async fn authorize_login(&self, identifier: &str) -> Result<bool, Supg005Error> {
        Ok(identifier != "suspended@arch.local")
    }

    async fn revoke_device_session(&self, _session_id: &str) -> Result<bool, Supg005Error> {
        if self.should_fail {
            return Err(Supg005Error::Internal("Failed to revoke session".to_string()));
        }
        Ok(true)
    }
}

/// `admin` reads, writes and approves; `writer` reads and writes; `reader` only reads.
struct MockAccess;

#[async_trait]
impl AccessControl for MockAccess {
    async fn authorize(
        &self,
        username: &str,
        program_code: &str,
        action: PermissionAction,
    ) -> Result<Option<Actor>, AccessError> {
        assert_eq!(program_code, "SUPG005");
        let allowed = match (username, action) {
            ("admin", _) => true,
            ("writer", PermissionAction::Read | PermissionAction::Write) => true,
            ("reader", PermissionAction::Read) => true,
            _ => false,
        };
        Ok(allowed.then(|| Actor { user_id: uuid::Uuid::nil(), username: username.to_string() }))
    }
}

fn router_for(username: &str, should_fail: bool) -> Router {
    let state = Supg005State {
        commands: Arc::new(MockSupg005Commands { should_fail }),
        access: Arc::new(MockAccess),
    };
    Router::new()
        .nest(SUPG005_PATH, supg005_routes_with_state(state))
        .layer(Extension(AuthenticatedUser {
            identity_id: "i".to_string(),
            session_id: "s".to_string(),
            email: Some(format!("{username}@arch.local")),
            traits: serde_json::json!({}),
        }))
}

fn create_test_router(should_fail: bool) -> Router {
    router_for("admin", should_fail)
}

#[tokio::test]
async fn test_permissions_read_write_approve() {
    let call = |user: &'static str, method: &'static str, uri: &'static str| async move {
        router_for(user, false)
            .oneshot(Request::builder().uri(uri).method(method).body(Body::empty()).unwrap())
            .await
            .unwrap()
            .status()
    };
    // Reading needs READ
    assert_eq!(call("stranger", "GET", "/su/supg005/users").await, StatusCode::FORBIDDEN);
    assert_eq!(call("reader", "GET", "/su/supg005/users").await, StatusCode::OK);
    // Suspend / reactivate need WRITE
    assert_eq!(call("reader", "POST", "/su/supg005/users/u1/suspend").await, StatusCode::FORBIDDEN);
    assert_eq!(call("writer", "POST", "/su/supg005/users/u1/suspend").await, StatusCode::OK);
    assert_eq!(call("writer", "POST", "/su/supg005/users/u1/reactivate").await, StatusCode::OK);
    // Revoking sessions needs APPROVE
    assert_eq!(call("writer", "DELETE", "/su/supg005/users/u1/sessions").await, StatusCode::FORBIDDEN);
    assert_eq!(call("writer", "DELETE", "/su/supg005/sessions/s1").await, StatusCode::FORBIDDEN);
    assert_eq!(call("admin", "DELETE", "/su/supg005/users/u1/sessions").await, StatusCode::OK);
    assert_eq!(call("admin", "DELETE", "/su/supg005/sessions/s1").await, StatusCode::OK);
}

#[tokio::test]
async fn test_list_users_success() {
    let app = create_test_router(false);

    let req = Request::builder()
        .uri("/su/supg005/users?page=1&limit=5")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let result: PaginatedResult<UserSessionOverviewDto> = serde_json::from_slice(&body).unwrap();
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.total, 1);
    assert_eq!(result.page, 1);
    assert_eq!(result.limit, 5);
    assert_eq!(result.items[0].username, "admin");
    assert_eq!(result.items[0].devices.len(), 1);
}

#[tokio::test]
async fn test_list_users_cbor_content_negotiation() {
    let app = create_test_router(false);

    let req = Request::builder()
        .uri("/su/supg005/users?page=1&limit=5")
        .method("GET")
        .header("Accept", "application/cbor")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers().get("content-type").unwrap(),
        "application/cbor"
    );

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let result: PaginatedResult<UserSessionOverviewDto> = infrastructure::wire::from_cbor_slice(&body).unwrap();
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.total, 1);
    assert_eq!(result.page, 1);
    assert_eq!(result.limit, 5);
    assert_eq!(result.items[0].username, "admin");
}

#[tokio::test]
async fn test_list_users_database_error() {
    let app = create_test_router(true);

    let req = Request::builder()
        .uri("/su/supg005/users")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_revoke_user_sessions_success() {
    let app = create_test_router(false);

    let req = Request::builder()
        .uri("/su/supg005/users/019ca1ec-0000-7000-8010-000000000001/sessions")
        .method("DELETE")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_revoke_device_session_success() {
    let app = create_test_router(false);

    let req = Request::builder()
        .uri("/su/supg005/sessions/dev-admin-01")
        .method("DELETE")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_suspend_and_reactivate_user() {
    for (path, fail, expected) in [
        ("suspend", false, StatusCode::OK),
        ("reactivate", false, StatusCode::OK),
        ("suspend", true, StatusCode::INTERNAL_SERVER_ERROR),
    ] {
        let app = create_test_router(fail);
        let req = Request::builder()
            .uri(format!("/su/supg005/users/019ca1ec-0000-7000-8010-000000000001/{path}"))
            .method("POST")
            .body(Body::empty())
            .unwrap();
        assert_eq!(app.oneshot(req).await.unwrap().status(), expected);
    }
}

#[tokio::test]
async fn test_login_guard_refuses_inactive_and_requires_secret() {
    // SAFETY: only this test touches KRATOS_WEBHOOK_SECRET.
    unsafe { std::env::set_var("KRATOS_WEBHOOK_SECRET", "test-secret") };
    let app = api_master::routers::kratos_hook_routes_with_commands(Arc::new(MockSupg005Commands {
        should_fail: false,
    }));

    let call = |identifier: &'static str, secret: Option<&'static str>| {
        let mut req = Request::builder()
            .method("POST")
            .uri("/login-guard")
            .header("content-type", "application/json");
        if let Some(s) = secret {
            req = req.header("x-webhook-secret", s);
        }
        req.body(Body::from(format!(r#"{{"identifier":"{identifier}"}}"#))).unwrap()
    };

    let ok = app.clone().oneshot(call("admin@arch.local", Some("test-secret"))).await.unwrap();
    assert_eq!(ok.status(), StatusCode::OK);

    let blocked = app.clone().oneshot(call("suspended@arch.local", Some("test-secret"))).await.unwrap();
    assert_eq!(blocked.status(), StatusCode::FORBIDDEN);

    let no_secret = app.oneshot(call("admin@arch.local", None)).await.unwrap();
    assert_eq!(no_secret.status(), StatusCode::UNAUTHORIZED);
}
