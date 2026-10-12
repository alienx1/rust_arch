//! Integration tests for SUPG003 router endpoints (`/su/supg003/...`) including authorization.
//! Located at `handler/api/master/tests/supg003_router_test.rs`.

use api_master::routers::{SUPG003_PATH, Supg003State, supg003_routes_with_state};
use application_master::features::su::access_control::{
    AccessControl, AccessError, Actor, PermissionAction,
};
use application_master::features::su::supg003::{
    CreatePermissionGroupCommand, GroupFilterQuery, MatrixRowDto, PermissionGroupDto, PermissionMatrixDto,
    SaveMatrixCommand, Supg003Commands, Supg003Error,
    UpdatePermissionGroupCommand,
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
        assert_eq!(program_code, "SUPG003");
        let allowed = matches!(
            (username, action),
            ("admin", _) | ("writer", PermissionAction::Read | PermissionAction::Write) | ("reader", PermissionAction::Read)
        );
        Ok(allowed.then(|| Actor { user_id: Uuid::nil(), username: username.to_string() }))
    }
}

struct MockCommands;

fn group(code: &str) -> PermissionGroupDto {
    PermissionGroupDto {
        id: Uuid::now_v7(),
        code: code.to_string(),
        active: true,
        user_count: 2,
        readable_programs: 21,
        is_protected: code == "ADMIN",
        row_version: 1,
    }
}

#[async_trait]
impl Supg003Commands for MockCommands {
    async fn list_groups(&self, f: GroupFilterQuery) -> Result<Vec<PermissionGroupDto>, Supg003Error> {
        Ok(vec![group(f.search.as_deref().unwrap_or("ADMIN"))])
    }
    async fn get_group(&self, _id: Uuid) -> Result<Option<PermissionGroupDto>, Supg003Error> {
        Ok(None)
    }
    async fn group_matrix(&self, id: Uuid) -> Result<Option<PermissionMatrixDto>, Supg003Error> {
        let mut g = group("HR");
        g.id = id;
        Ok(Some(PermissionMatrixDto {
            group: g,
            rows: vec![MatrixRowDto {
                program_id: Uuid::now_v7(),
                system_code: "DB".to_string(),
                module_code: "DBMT".to_string(),
                program_code: "DBPG003".to_string(),
                name_th: "พนักงาน".to_string(),
                name_en: "Employees".to_string(),
                parent_id: None,
                depth: 0,
                is_folder: false,
                program_active: true,
                read: true,
                write: false,
                approve: false,
            }],
        }))
    }
    async fn create_group(&self, cmd: CreatePermissionGroupCommand, actor: Option<Uuid>) -> Result<PermissionGroupDto, Supg003Error> {
        assert_eq!(actor, Some(Uuid::nil()), "the authorized user is recorded as the actor");
        if cmd.code == "DUP" {
            return Err(Supg003Error::DuplicateCode("DUP".to_string()));
        }
        Ok(group(&cmd.code))
    }
    async fn update_group(&self, _id: Uuid, cmd: UpdatePermissionGroupCommand, _a: Option<Uuid>) -> Result<PermissionGroupDto, Supg003Error> {
        if cmd.row_version != 1 {
            return Err(Supg003Error::VersionConflict);
        }
        Ok(group("HR"))
    }
    async fn save_matrix(&self, id: Uuid, cmd: SaveMatrixCommand, _a: Option<Uuid>) -> Result<PermissionMatrixDto, Supg003Error> {
        if cmd.grants.iter().any(|g| g.write && !g.read) {
            return Err(Supg003Error::Validation("WRITE needs READ".to_string()));
        }
        self.group_matrix(id).await.map(|m| m.unwrap())
    }
    async fn delete_group(&self, id: Uuid) -> Result<(), Supg003Error> {
        // The mock treats the all-ones id as a group that still has users
        if id == Uuid::from_u128(1) {
            return Err(Supg003Error::InUse("The group still has 2 user(s)".to_string()));
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
    let state = Supg003State { commands: Arc::new(MockCommands), access: Arc::new(MockAccess) };
    Router::new()
        .nest(SUPG003_PATH, supg003_routes_with_state(state))
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

async fn status_and_json(resp: axum::response::Response) -> (StatusCode, serde_json::Value) {
    let status = resp.status();
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null))
}

#[tokio::test]
async fn test_reads_need_read_permission() {
    let (status, body) = status_and_json(router("reader").oneshot(get("/su/supg003/groups?search=HR")).await.unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body[0]["code"], "HR");

    for uri in ["/su/supg003/groups", &format!("/su/supg003/groups/{}/matrix", Uuid::now_v7())] {
        let denied = router("stranger").oneshot(get(uri)).await.unwrap();
        assert_eq!(denied.status(), StatusCode::FORBIDDEN, "{uri}");
    }
}

#[tokio::test]
async fn test_matrix_is_returned_with_tree_rows() {
    let id = Uuid::now_v7();
    let (status, body) = status_and_json(
        router("reader").oneshot(get(&format!("/su/supg003/groups/{id}/matrix"))).await.unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["group"]["id"], id.to_string());
    assert_eq!(body["rows"][0]["program_code"], "DBPG003");
    assert_eq!(body["rows"][0]["read"], true);
}

#[tokio::test]
async fn test_matrix_route_has_no_scope_and_save_passes_through() {
    let id = Uuid::now_v7();
    let program = Uuid::now_v7();
    let response = router("admin")
        .oneshot(json_request(
            "PUT",
            &format!("/su/supg003/groups/{id}/matrix"),
            serde_json::json!({ "grants": [{ "program_id": program, "read": true, "write": false, "approve": false }] }),
        ))
        .await
        .unwrap();
    let (status, body) = status_and_json(response).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["group"]["id"], id.to_string());
    assert!(body.get("scope_id").is_none(), "the matrix has no scope any more");
    assert!(body["group"].get("scopes").is_none());

    let gone = router("reader").oneshot(get("/su/supg003/scope-options")).await.unwrap();
    assert_eq!(gone.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_saving_a_matrix_needs_write_and_maps_validation_errors() {
    let id = Uuid::now_v7();
    let save = |read: bool, write: bool| {
        json_request(
            "PUT",
            &format!("/su/supg003/groups/{id}/matrix"),
            serde_json::json!({ "grants": [{ "program_id": Uuid::now_v7(), "read": read, "write": write, "approve": false }] }),
        )
    };

    assert_eq!(router("reader").oneshot(save(true, false)).await.unwrap().status(), StatusCode::FORBIDDEN);
    assert_eq!(router("admin").oneshot(save(true, true)).await.unwrap().status(), StatusCode::OK);
    assert_eq!(router("admin").oneshot(save(false, true)).await.unwrap().status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_group_write_status_codes() {
    let app = router("admin");
    let create = |code: &str| {
        json_request(
            "POST",
            "/su/supg003/groups",
            serde_json::json!({ "code": code, "active": true }),
        )
    };
    assert_eq!(app.clone().oneshot(create("HR")).await.unwrap().status(), StatusCode::OK);
    assert_eq!(app.clone().oneshot(create("DUP")).await.unwrap().status(), StatusCode::CONFLICT);

    let stale = app
        .oneshot(json_request(
            "PUT",
            &format!("/su/supg003/groups/{}", Uuid::now_v7()),
            serde_json::json!({ "active": false, "row_version": 5 }),
        ))
        .await
        .unwrap();
    assert_eq!(stale.status(), StatusCode::PRECONDITION_FAILED);
}

#[tokio::test]
async fn test_delete_group_needs_approve_and_maps_in_use() {
    let delete = |id: Uuid| {
        Request::builder().uri(format!("/su/supg003/groups/{id}")).method("DELETE").body(Body::empty()).unwrap()
    };

    assert_eq!(router("reader").oneshot(delete(Uuid::now_v7())).await.unwrap().status(), StatusCode::FORBIDDEN);
    assert_eq!(router("writer").oneshot(delete(Uuid::now_v7())).await.unwrap().status(), StatusCode::FORBIDDEN);
    assert_eq!(router("admin").oneshot(delete(Uuid::now_v7())).await.unwrap().status(), StatusCode::NO_CONTENT);

    let (status, body) = status_and_json(router("admin").oneshot(delete(Uuid::from_u128(1))).await.unwrap()).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "IN_USE");
}
