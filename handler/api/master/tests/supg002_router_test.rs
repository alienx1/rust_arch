//! Integration tests for the SUPG002 router: the menu tree (`/su/supg002/menu`) and program
//! management (`/su/supg002/programs`) including authorization.
//! Located at `handler/api/master/tests/supg002_router_test.rs`.

use api_master::routers::{SUPG002_PATH, Supg002State, supg002_routes_with_state, username_of};
use application_master::features::su::access_control::{
    AccessControl, AccessError, Actor, PermissionAction,
};
use application_master::features::su::supg002::{
    CreateProgramCommand, MenuNodeDto, ProgramDto, ProgramFilterQuery, ProgramOptionsDto,
    ProgramPageDto, Supg002Commands, Supg002Error, UpdateProgramCommand,
};
use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum::{Extension, Router};
use infrastructure::middleware::AuthenticatedUser;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

/// `admin` may read, write and approve, `writer` may read and write, `reader` may only read.
struct MockAccess;

#[async_trait]
impl AccessControl for MockAccess {
    async fn authorize(
        &self,
        username: &str,
        program_code: &str,
        action: PermissionAction,
    ) -> Result<Option<Actor>, AccessError> {
        assert_eq!(program_code, "SUPG002");
        let allowed = matches!(
            (username, action),
            ("admin", _) | ("writer", PermissionAction::Read | PermissionAction::Write) | ("reader", PermissionAction::Read)
        );
        Ok(allowed.then(|| Actor { user_id: Uuid::nil(), username: username.to_string() }))
    }
}

struct MockCommands;

fn program(code: &str, row_version: i64) -> ProgramDto {
    ProgramDto {
        id: Uuid::now_v7(),
        system_id: Uuid::now_v7(),
        system_code: "DB".to_string(),
        module_id: Uuid::now_v7(),
        module_code: "DBMT".to_string(),
        parent_id: None,
        parent_code: String::new(),
        code: code.to_string(),
        path: Some("/db/dbpg006".to_string()),
        icon: None,
        sort_order: 6,
        active: true,
        name_th: "ใหม่".to_string(),
        name_en: "New".to_string(),
        child_count: 0,
        is_protected: false,
        row_version,
    }
}

#[async_trait]
impl Supg002Commands for MockCommands {
    async fn get_menu_tree(&self, username: &str) -> Result<Vec<MenuNodeDto>, Supg002Error> {
        Ok(vec![MenuNodeDto {
            id: Uuid::now_v7(),
            code: username.to_uppercase(),
            path: Some("/".to_string()),
            icon: None,
            label_th: "ภาพรวมระบบ".to_string(),
            label_en: "Dashboard".to_string(),
            sort_order: 1,
            children: vec![],
        }])
    }

    async fn list_programs(&self, f: ProgramFilterQuery) -> Result<ProgramPageDto, Supg002Error> {
        Ok(ProgramPageDto {
            items: vec![program(f.search.as_deref().unwrap_or("DBPG006"), 1)],
            total: 21,
            page: f.page.unwrap_or(1),
            limit: f.limit.unwrap_or(25),
        })
    }

    async fn get_program(&self, _id: Uuid) -> Result<Option<ProgramDto>, Supg002Error> {
        Ok(None)
    }

    async fn program_options(&self) -> Result<ProgramOptionsDto, Supg002Error> {
        Ok(ProgramOptionsDto { systems: vec![], modules: vec![], programs: vec![] })
    }

    async fn create_program(&self, cmd: CreateProgramCommand, actor: Option<Uuid>) -> Result<ProgramDto, Supg002Error> {
        assert_eq!(actor, Some(Uuid::nil()), "the authorized user is recorded as the actor");
        if cmd.code == "DUP" {
            return Err(Supg002Error::DuplicateCode("DUP".to_string()));
        }
        Ok(program(&cmd.code, 1))
    }

    async fn update_program(&self, _id: Uuid, cmd: UpdateProgramCommand, _a: Option<Uuid>) -> Result<ProgramDto, Supg002Error> {
        if cmd.row_version != 1 {
            return Err(Supg002Error::VersionConflict);
        }
        if !cmd.active {
            return Err(Supg002Error::Validation("protected".to_string()));
        }
        Ok(program("DBPG006", 2))
    }

    async fn delete_program(&self, id: Uuid) -> Result<(), Supg002Error> {
        // The nil id stands for a program that still has child programs
        if id.is_nil() {
            return Err(Supg002Error::InUse("still referenced".to_string()));
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

fn router(user: Option<AuthenticatedUser>) -> Router {
    let state = Supg002State { commands: Arc::new(MockCommands), access: Arc::new(MockAccess) };
    let app = Router::new().nest(SUPG002_PATH, supg002_routes_with_state(state));
    match user {
        Some(u) => app.layer(Extension(u)),
        None => app,
    }
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

async fn status_and_json(resp: axum::response::Response) -> (StatusCode, serde_json::Value) {
    let status = resp.status();
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null))
}

#[test]
fn test_username_prefers_trait_then_email_local_part() {
    let with_trait = AuthenticatedUser {
        identity_id: "i".to_string(),
        session_id: "s".to_string(),
        email: Some("a@arch.local".to_string()),
        traits: serde_json::json!({ "username": "boss" }),
    };
    assert_eq!(username_of(&with_trait).as_deref(), Some("boss"));
    assert_eq!(username_of(&user("admin")).as_deref(), Some("admin"));

    let none = AuthenticatedUser {
        identity_id: "i".to_string(),
        session_id: "s".to_string(),
        email: None,
        traits: serde_json::json!({}),
    };
    assert_eq!(username_of(&none), None);
}

#[tokio::test]
async fn test_menu_needs_no_program_permission_and_is_built_for_the_account() {
    // `stranger` holds no SUPG002 permission at all, yet gets their own menu
    let (status, body) = status_and_json(
        router(Some(user("stranger"))).oneshot(get("/su/supg002/menu")).await.unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body[0]["code"], "STRANGER");
}

#[tokio::test]
async fn test_program_reads_need_read_and_writes_need_write() {
    let denied = router(Some(user("stranger"))).oneshot(get("/su/supg002/programs")).await.unwrap();
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    let (status, body) = status_and_json(
        router(Some(user("reader")))
            .oneshot(get("/su/supg002/programs?search=X1&page=2&limit=10"))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"][0]["code"], "X1");
    assert_eq!((body["page"].as_u64(), body["limit"].as_u64()), (Some(2), Some(10)));

    let create = |code: &str| {
        json_request(
            "POST",
            "/su/supg002/programs",
            serde_json::json!({
                "system_id": Uuid::now_v7(), "module_id": Uuid::now_v7(), "parent_id": null,
                "code": code, "path": "/db/dbpg006", "icon": null, "sort_order": 6, "active": true,
                "name_th": "ใหม่", "name_en": ""
            }),
        )
    };
    assert_eq!(
        router(Some(user("reader"))).oneshot(create("NEW")).await.unwrap().status(),
        StatusCode::FORBIDDEN,
        "READ alone must not allow a write"
    );
    assert_eq!(
        router(Some(user("admin"))).oneshot(create("NEW")).await.unwrap().status(),
        StatusCode::OK
    );
    assert_eq!(
        router(Some(user("admin"))).oneshot(create("DUP")).await.unwrap().status(),
        StatusCode::CONFLICT
    );
}

#[tokio::test]
async fn test_update_error_status_codes() {
    let app = router(Some(user("admin")));
    let update = |row_version: i64, active: bool| {
        json_request(
            "PUT",
            &format!("/su/supg002/programs/{}", Uuid::now_v7()),
            serde_json::json!({
                "parent_id": null, "path": null, "icon": null, "sort_order": 1, "active": active,
                "name_th": "ชื่อ", "name_en": "", "row_version": row_version
            }),
        )
    };
    assert_eq!(app.clone().oneshot(update(1, true)).await.unwrap().status(), StatusCode::OK);
    assert_eq!(app.clone().oneshot(update(7, true)).await.unwrap().status(), StatusCode::PRECONDITION_FAILED);
    assert_eq!(app.oneshot(update(1, false)).await.unwrap().status(), StatusCode::BAD_REQUEST);
}

fn delete(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).method("DELETE").body(Body::empty()).unwrap()
}

#[tokio::test]
async fn test_delete_program_returns_no_content() {
    let resp = router(Some(user("admin")))
        .oneshot(delete(&format!("/su/supg002/programs/{}", Uuid::now_v7())))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_of_a_referenced_program_is_a_conflict() {
    let resp = router(Some(user("admin")))
        .oneshot(delete(&format!("/su/supg002/programs/{}", Uuid::nil())))
        .await
        .unwrap();
    let (status, body) = status_and_json(resp).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "IN_USE");
}

#[tokio::test]
async fn test_delete_program_needs_approve_permission() {
    for who in ["reader", "writer"] {
        let resp = router(Some(user(who)))
            .oneshot(delete(&format!("/su/supg002/programs/{}", Uuid::now_v7())))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN, "{who}");
    }
}
