//! Integration tests for SUPG004 router endpoints (`/su/supg004/...`): authorization, the
//! ADMIN-group approval flag and error mapping.
//! Located at `handler/api/master/tests/supg004_router_test.rs`.

use api_master::routers::{SUPG004_PATH, Supg004State, supg004_routes_with_state};
use application_master::features::su::account_security::PasswordMailResult;
use application_master::features::su::access_control::{
    AccessControl, AccessError, Actor, PermissionAction,
};
use application_master::features::su::supg004::{
    CreateUserCommand, CreateUserResultDto, ScopeOptionDto, ScopeOptionsDto, UserAssignmentDto, EmployeeOptionDto, EmployeeOptionsQuery,
    GroupOptionDto, Supg004Commands, Supg004Error, UpdateUserCommand,
    UserAccountDto, UserFilterQuery, UserPageDto,
};
use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum::{Extension, Router};
use infrastructure::middleware::AuthenticatedUser;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

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
        assert_eq!(program_code, "SUPG004");
        let allowed = match (username, action) {
            ("admin", _) => true,
            ("writer", PermissionAction::Read | PermissionAction::Write) => true,
            ("reader", PermissionAction::Read) => true,
            _ => false,
        };
        Ok(allowed.then(|| Actor { user_id: Uuid::nil(), username: username.to_string() }))
    }
}

struct MockCommands;

fn account(username: &str) -> UserAccountDto {
    UserAccountDto {
        id: Uuid::now_v7(),
        username: username.to_string(),
        employee_id: Uuid::now_v7(),
        employee_code: "E001".to_string(),
        full_name: "สมชาย ใจดี".to_string(),
        department_name: "ไอที".to_string(),
        company_name: "บริษัท 1".to_string(),
        group_id: Uuid::now_v7(),
        group_code: "OPERATOR".to_string(),
        assignments: vec![UserAssignmentDto {
            group_id: Uuid::now_v7(),
            group_code: "OPERATOR".to_string(),
            scope_level: "branch".to_string(),
            scope_id: Uuid::now_v7(),
            scope_code: "B1".to_string(),
            scope_name: "สาขา 1".to_string(),
            scope_path: "ORG › C1 › B1".to_string(),
        }],
        active: true,
        must_change_password: false,
        created_time: "2026-10-09T00:00:00+00:00".to_string(),
        row_version: 1,
    }
}

#[async_trait]
impl Supg004Commands for MockCommands {
    async fn list_users(&self, f: UserFilterQuery) -> Result<UserPageDto, Supg004Error> {
        Ok(UserPageDto {
            items: vec![account(f.search.as_deref().unwrap_or("somchai"))],
            total: 3,
            page: f.page.unwrap_or(1),
            limit: f.limit.unwrap_or(25),
        })
    }
    async fn get_user(&self, _id: Uuid) -> Result<Option<UserAccountDto>, Supg004Error> {
        Ok(None)
    }
    async fn employee_options(&self, _q: EmployeeOptionsQuery) -> Result<Vec<EmployeeOptionDto>, Supg004Error> {
        Ok(vec![])
    }
    async fn group_options(&self) -> Result<Vec<GroupOptionDto>, Supg004Error> {
        Ok(vec![])
    }
    async fn scope_options(&self) -> Result<ScopeOptionsDto, Supg004Error> {
        Ok(ScopeOptionsDto {
            organizations: vec![],
            companies: vec![],
            branches: vec![ScopeOptionDto {
                id: Uuid::now_v7(),
                code: "B1".to_string(),
                name_th: "สาขา 1".to_string(),
                name_en: "Branch 1".to_string(),
                parent_id: Some(Uuid::now_v7()),
            }],
            departments: vec![],
        })
    }
    async fn create_user(
        &self,
        cmd: CreateUserCommand,
        actor: Option<Uuid>,
        can_assign_admin: bool,
    ) -> Result<CreateUserResultDto, Supg004Error> {
        assert_eq!(actor, Some(Uuid::nil()), "the authorized user is recorded as the actor");
        assert!(!cmd.assignments.is_empty(), "the assignments reach the executor");
        match cmd.username.as_str() {
            "dup" => return Err(Supg004Error::Duplicate("dup".to_string())),
            "kratos-down" => return Err(Supg004Error::IdentityProvider("unreachable".to_string())),
            // The mock treats this username as "wants the ADMIN group"
            "wants-admin" if !can_assign_admin => {
                return Err(Supg004Error::NotAllowed("ADMIN needs APPROVE".to_string()));
            }
            _ => {}
        }
        Ok(CreateUserResultDto {
            user: account(&cmd.username),
            password_mail: PasswordMailResult {
                sent: true,
                sent_to: Some("n***@arch.local".to_string()),
                mail_error: None,
                recovery_link: None,
            },
        })
    }
    async fn update_user(&self, _id: Uuid, cmd: UpdateUserCommand, _a: Option<Uuid>, _admin: bool) -> Result<UserAccountDto, Supg004Error> {
        assert_eq!(cmd.assignments.len(), 2, "the assignments reach the executor");
        if cmd.row_version != 1 {
            return Err(Supg004Error::VersionConflict);
        }
        Ok(account("somchai"))
    }
    async fn send_password_mail(&self, id: Uuid) -> Result<PasswordMailResult, Supg004Error> {
        if id == Uuid::nil() {
            return Err(Supg004Error::NotFound(id.to_string()));
        }
        Ok(PasswordMailResult { sent: true, sent_to: Some("s***@arch.local".to_string()), mail_error: None, recovery_link: None })
    }
    async fn force_password_change(&self, _id: Uuid) -> Result<PasswordMailResult, Supg004Error> {
        // The mail server is down: the link comes back for the administrator to hand over
        Ok(PasswordMailResult {
            sent: false,
            sent_to: Some("s***@arch.local".to_string()),
            mail_error: Some("no mail server".to_string()),
            recovery_link: Some("http://localhost:3000/recovery?token=xyz".to_string()),
        })
    }
    async fn delete_user(&self, id: Uuid, actor: Option<Uuid>) -> Result<(), Supg004Error> {
        assert_eq!(actor, Some(Uuid::nil()), "the authorized user is passed as the actor");
        // The mock treats the nil id as the caller's own account and the all-ones id as referenced
        if id == Uuid::nil() {
            return Err(Supg004Error::NotAllowed("You cannot delete your own account".to_string()));
        }
        if id == Uuid::from_u128(1) {
            return Err(Supg004Error::InUse("The account is still referenced".to_string()));
        }
        Ok(())
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

fn router(username: &str) -> Router {
    let state = Supg004State { commands: Arc::new(MockCommands), access: Arc::new(MockAccess) };
    Router::new()
        .nest(SUPG004_PATH, supg004_routes_with_state(state))
        .layer(Extension(user(username)))
}

fn get(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).method("GET").body(Body::empty()).unwrap()
}

fn json_request(method: &str, uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .method(method)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn create_body(username: &str) -> serde_json::Value {
    serde_json::json!({
        "employee_id": Uuid::now_v7(), "username": username, "email": format!("{username}@corp.example"),
        "assignments": [
            { "permission_group_id": Uuid::now_v7(), "scope_level": "branch", "scope_id": Uuid::now_v7() }
        ],
        "active": true
    })
}

fn two_assignments() -> serde_json::Value {
    serde_json::json!([
        { "permission_group_id": Uuid::now_v7(), "scope_level": "branch", "scope_id": Uuid::now_v7() },
        { "permission_group_id": Uuid::now_v7(), "scope_level": "department", "scope_id": Uuid::now_v7() }
    ])
}

async fn status_and_json(resp: axum::response::Response) -> (StatusCode, serde_json::Value) {
    let status = resp.status();
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null))
}

#[tokio::test]
async fn test_reads_need_read_and_paging_is_parsed() {
    let (status, body) = status_and_json(
        router("reader").oneshot(get("/su/supg004/users?search=abc&page=2&limit=10")).await.unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"][0]["username"], "abc");
    assert_eq!((body["page"].as_u64(), body["limit"].as_u64()), (Some(2), Some(10)));

    for uri in ["/su/supg004/users", "/su/supg004/options/groups", "/su/supg004/options/employees"] {
        assert_eq!(router("stranger").oneshot(get(uri)).await.unwrap().status(), StatusCode::FORBIDDEN, "{uri}");
    }
}

#[tokio::test]
async fn test_scope_options_need_read_and_return_all_levels() {
    let (status, body) = status_and_json(router("reader").oneshot(get("/su/supg004/scope-options")).await.unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["branches"][0]["code"], "B1");
    assert!(body["organizations"].is_array() && body["departments"].is_array());
    assert_eq!(router("stranger").oneshot(get("/su/supg004/scope-options")).await.unwrap().status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_update_with_assignments_needs_write() {
    let put = |row_version: i64| {
        json_request(
            "PUT",
            &format!("/su/supg004/users/{}", Uuid::now_v7()),
            serde_json::json!({ "assignments": two_assignments(), "active": true, "row_version": row_version }),
        )
    };
    assert_eq!(router("reader").oneshot(put(1)).await.unwrap().status(), StatusCode::FORBIDDEN);
    let (status, body) = status_and_json(router("writer").oneshot(put(1)).await.unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["assignments"][0]["scope_path"], "ORG › C1 › B1");
}

#[tokio::test]
async fn test_writes_need_write_and_the_new_account_gets_its_password_mail() {
    let create = |u: &str| json_request("POST", "/su/supg004/users", create_body(u));

    assert_eq!(router("reader").oneshot(create("new")).await.unwrap().status(), StatusCode::FORBIDDEN);

    let (status, body) = status_and_json(router("writer").oneshot(create("new")).await.unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user"]["username"], "new");
    assert_eq!(body["password_mail"]["sent"], true);
    assert!(body["password_mail"]["recovery_link"].is_null(), "the link is not handed out when the mail went");

    // Mailing a link and forcing a change are writes
    let post = |what: &str| json_request("POST", &format!("/su/supg004/users/{}/{what}", Uuid::now_v7()), serde_json::json!({}));
    for what in ["password-mail", "force-password-change"] {
        assert_eq!(router("reader").oneshot(post(what)).await.unwrap().status(), StatusCode::FORBIDDEN, "{what} is a write");
    }
    let (status, mailed) = status_and_json(router("writer").oneshot(post("password-mail")).await.unwrap()).await;
    assert_eq!((status, &mailed["sent"], &mailed["sent_to"]), (StatusCode::OK, &serde_json::json!(true), &serde_json::json!("s***@arch.local")));

    // The mail server is down: the administrator gets the link and the reason
    let (status, forced) = status_and_json(router("writer").oneshot(post("force-password-change")).await.unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(forced["sent"], false);
    assert!(forced["recovery_link"].as_str().unwrap().contains("recovery"));
    assert_eq!(forced["mail_error"], "no mail server");

    // An unknown account is a 404
    let missing = router("writer")
        .oneshot(json_request("POST", &format!("/su/supg004/users/{}/password-mail", Uuid::nil()), serde_json::json!({})))
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_admin_group_assignment_depends_on_the_approve_permission() {
    let create = || json_request("POST", "/su/supg004/users", create_body("wants-admin"));

    // `writer` lacks APPROVE: the executor is told so and refuses (403 NOT_ALLOWED)
    let (status, body) = status_and_json(router("writer").oneshot(create()).await.unwrap()).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "NOT_ALLOWED");

    // `admin` holds APPROVE
    assert_eq!(router("admin").oneshot(create()).await.unwrap().status(), StatusCode::OK);
}

#[tokio::test]
async fn test_error_status_codes() {
    let app = router("writer");
    let create = |u: &str| json_request("POST", "/su/supg004/users", create_body(u));
    assert_eq!(app.clone().oneshot(create("dup")).await.unwrap().status(), StatusCode::CONFLICT);
    let (status, body) = status_and_json(app.clone().oneshot(create("kratos-down")).await.unwrap()).await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(body["code"], "IDENTITY_PROVIDER_ERROR");

    let stale = app
        .oneshot(json_request(
            "PUT",
            &format!("/su/supg004/users/{}", Uuid::now_v7()),
            serde_json::json!({ "assignments": two_assignments(), "active": true, "row_version": 9 }),
        ))
        .await
        .unwrap();
    assert_eq!(stale.status(), StatusCode::PRECONDITION_FAILED);
}

#[tokio::test]
async fn test_delete_user_needs_approve_and_maps_in_use_and_self_delete() {
    let delete = |id: Uuid| {
        Request::builder().uri(format!("/su/supg004/users/{id}")).method("DELETE").body(Body::empty()).unwrap()
    };

    assert_eq!(router("reader").oneshot(delete(Uuid::now_v7())).await.unwrap().status(), StatusCode::FORBIDDEN);
    assert_eq!(router("writer").oneshot(delete(Uuid::now_v7())).await.unwrap().status(), StatusCode::FORBIDDEN);
    assert_eq!(router("admin").oneshot(delete(Uuid::now_v7())).await.unwrap().status(), StatusCode::NO_CONTENT);

    let (status, body) = status_and_json(router("admin").oneshot(delete(Uuid::from_u128(1))).await.unwrap()).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "IN_USE");

    let (status, body) = status_and_json(router("admin").oneshot(delete(Uuid::nil())).await.unwrap()).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "NOT_ALLOWED");
}
