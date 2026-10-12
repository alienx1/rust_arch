//! Integration tests for SUPG001 router endpoints (`/su/supg001/...`) including authorization.
//! Located at `handler/api/master/tests/supg001_router_test.rs`.

use api_master::routers::{SUPG001_PATH, Supg001State, supg001_routes_with_state};
use application_master::features::su::access_control::{
    AccessControl, AccessError, Actor, PermissionAction,
};
use application_master::features::su::supg001::{
    CreateModuleCommand, CreateSystemCommand, ModuleDto, ModuleFilterQuery, Supg001Commands,
    Supg001Error, SystemDto, SystemFilterQuery, UpdateUnitCommand,
};
use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum::{Extension, Router};
use infrastructure::middleware::AuthenticatedUser;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

/// `admin` may read, write and approve, `writer` may read and write, `reader` may only read, everyone else nothing.
struct MockAccess;

#[async_trait]
impl AccessControl for MockAccess {
    async fn authorize(
        &self,
        username: &str,
        program_code: &str,
        action: PermissionAction,
    ) -> Result<Option<Actor>, AccessError> {
        assert_eq!(program_code, "SUPG001");
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

impl MockCommands {
    fn system(code: &str, row_version: i64) -> SystemDto {
        SystemDto {
            id: Uuid::now_v7(),
            code: code.to_string(),
            active: true,
            name_th: "ความปลอดภัย".to_string(),
            name_en: "Security".to_string(),
            module_count: 1,
            program_count: 6,
            is_protected: code == "SU",
            row_version,
        }
    }
}

#[async_trait]
impl Supg001Commands for MockCommands {
    async fn list_systems(&self, _f: SystemFilterQuery) -> Result<Vec<SystemDto>, Supg001Error> {
        Ok(vec![Self::system("SU", 1)])
    }
    async fn get_system(&self, _id: Uuid) -> Result<Option<SystemDto>, Supg001Error> {
        Ok(None)
    }
    async fn list_modules(&self, f: ModuleFilterQuery) -> Result<Vec<ModuleDto>, Supg001Error> {
        // Echo that the system filter arrived: no filter => no rows
        Ok(if f.system_id.is_some() {
            vec![ModuleDto {
                id: Uuid::now_v7(),
                system_id: f.system_id.unwrap(),
                system_code: "SU".to_string(),
                system_name: "ความปลอดภัย".to_string(),
                code: "SUMT".to_string(),
                active: true,
                name_th: "จัดการสิทธิ์".to_string(),
                name_en: "Security".to_string(),
                program_count: 6,
                is_protected: true,
                row_version: 1,
            }]
        } else {
            vec![]
        })
    }
    async fn get_module(&self, _id: Uuid) -> Result<Option<ModuleDto>, Supg001Error> {
        Ok(None)
    }
    async fn create_system(&self, cmd: CreateSystemCommand, actor: Option<Uuid>) -> Result<SystemDto, Supg001Error> {
        assert_eq!(actor, Some(Uuid::nil()), "the authorized user is recorded as the actor");
        if cmd.code == "DUP" {
            return Err(Supg001Error::DuplicateCode("DUP".to_string()));
        }
        Ok(Self::system(&cmd.code, 1))
    }
    async fn update_system(&self, _id: Uuid, cmd: UpdateUnitCommand, _a: Option<Uuid>) -> Result<SystemDto, Supg001Error> {
        if cmd.row_version != 1 {
            return Err(Supg001Error::VersionConflict);
        }
        Ok(Self::system("DB", 2))
    }
    async fn create_module(&self, _cmd: CreateModuleCommand, _a: Option<Uuid>) -> Result<ModuleDto, Supg001Error> {
        Err(Supg001Error::NotFound("system".to_string()))
    }
    async fn update_module(&self, _id: Uuid, _cmd: UpdateUnitCommand, _a: Option<Uuid>) -> Result<ModuleDto, Supg001Error> {
        Err(Supg001Error::Validation("protected".to_string()))
    }
    async fn delete_system(&self, id: Uuid) -> Result<(), Supg001Error> {
        // The nil id stands for a system that is still referenced
        if id.is_nil() {
            return Err(Supg001Error::InUse("still referenced".to_string()));
        }
        Ok(())
    }
    async fn delete_module(&self, id: Uuid) -> Result<(), Supg001Error> {
        if id.is_nil() {
            return Err(Supg001Error::InUse("still referenced".to_string()));
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

fn router(username: Option<&str>) -> Router {
    let state = Supg001State { commands: Arc::new(MockCommands), access: Arc::new(MockAccess) };
    let app = Router::new().nest(SUPG001_PATH, supg001_routes_with_state(state));
    match username {
        Some(u) => app.layer(Extension(user(u))),
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

#[tokio::test]
async fn test_reads_need_read_permission() {
    let ok = router(Some("reader")).oneshot(get("/su/supg001/systems")).await.unwrap();
    let (status, body) = status_and_json(ok).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body[0]["code"], "SU");

    let denied = router(Some("stranger")).oneshot(get("/su/supg001/systems")).await.unwrap();
    let (status, body) = status_and_json(denied).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "FORBIDDEN");
}

#[tokio::test]
async fn test_writes_need_write_permission() {
    let create = |code: &str| {
        json_request(
            "POST",
            "/su/supg001/systems",
            serde_json::json!({ "code": code, "active": true, "name_th": "ใหม่", "name_en": "" }),
        )
    };

    let reader = router(Some("reader")).oneshot(create("NEW")).await.unwrap();
    assert_eq!(reader.status(), StatusCode::FORBIDDEN, "READ alone must not allow a write");

    let admin = router(Some("admin")).oneshot(create("NEW")).await.unwrap();
    assert_eq!(admin.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_write_error_status_codes() {
    let app = router(Some("admin"));
    let dup = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/su/supg001/systems",
            serde_json::json!({ "code": "DUP", "active": true, "name_th": "ซ้ำ", "name_en": "" }),
        ))
        .await
        .unwrap();
    assert_eq!(dup.status(), StatusCode::CONFLICT);

    let stale = app
        .oneshot(json_request(
            "PUT",
            &format!("/su/supg001/systems/{}", Uuid::now_v7()),
            serde_json::json!({ "active": true, "name_th": "ชื่อ", "name_en": "", "row_version": 9 }),
        ))
        .await
        .unwrap();
    assert_eq!(stale.status(), StatusCode::PRECONDITION_FAILED);
}

#[tokio::test]
async fn test_module_list_passes_the_system_filter() {
    let system = Uuid::now_v7();
    let resp = router(Some("reader"))
        .oneshot(get(&format!("/su/supg001/modules?system_id={system}")))
        .await
        .unwrap();
    let (status, body) = status_and_json(resp).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body[0]["code"], "SUMT");
    assert_eq!(body[0]["system_id"], system.to_string());
}

fn delete(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).method("DELETE").body(Body::empty()).unwrap()
}

#[tokio::test]
async fn test_delete_returns_no_content() {
    for kind in ["systems", "modules"] {
        let resp = router(Some("admin"))
            .oneshot(delete(&format!("/su/supg001/{kind}/{}", Uuid::now_v7())))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT, "{kind}");
    }
}

#[tokio::test]
async fn test_delete_of_a_referenced_row_is_a_conflict() {
    for kind in ["systems", "modules"] {
        let resp = router(Some("admin"))
            .oneshot(delete(&format!("/su/supg001/{kind}/{}", Uuid::nil())))
            .await
            .unwrap();
        let (status, body) = status_and_json(resp).await;
        assert_eq!(status, StatusCode::CONFLICT, "{kind}");
        assert_eq!(body["code"], "IN_USE");
    }
}

#[tokio::test]
async fn test_delete_needs_approve_permission() {
    for kind in ["systems", "modules"] {
        for who in ["reader", "writer"] {
            let resp = router(Some(who))
                .oneshot(delete(&format!("/su/supg001/{kind}/{}", Uuid::now_v7())))
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::FORBIDDEN, "{kind} {who}");
        }
    }
}
