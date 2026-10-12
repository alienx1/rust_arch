//! Integration tests for DBPG004 router endpoints (`/db/dbpg004/...`).
//! Located at `handler/api/master/tests/dbpg004_router_test.rs`.

use api_master::routers::{DBPG004_PATH, Dbpg004State, dbpg004_routes_with_state};
use application_master::features::db::dbpg004::{
    CreateLookupCommand, Dbpg004Commands, Dbpg004Error, Dbpg004SummaryDto, LookupCategoryDto,
    LookupDto, LookupFilterQuery, LookupPageDto, UpdateLookupCommand,
};
use application_master::features::su::access_control::{
    AccessControl, AccessError, Actor, PermissionAction,
};
use async_trait::async_trait;
use axum::{Extension, Router};
use infrastructure::middleware::AuthenticatedUser;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

struct MockDbpg004Commands {
    lookup_id: Uuid,
}

impl MockDbpg004Commands {
    fn lookup(&self, code: &str, row_version: i64) -> LookupDto {
        LookupDto {
            id: self.lookup_id,
            category: "gender".to_string(),
            code: code.to_string(),
            active: true,
            name_th: "ชาย".to_string(),
            name_en: "Male".to_string(),
            is_protected: false,
            row_version,
        }
    }
}

#[async_trait]
impl Dbpg004Commands for MockDbpg004Commands {
    async fn list_categories(&self) -> Result<Vec<LookupCategoryDto>, Dbpg004Error> {
        Ok(vec![LookupCategoryDto {
            category: "gender".to_string(),
            total: 3,
            active: 3,
            is_protected: false,
        }])
    }

    async fn list_lookups(&self, filter: LookupFilterQuery) -> Result<LookupPageDto, Dbpg004Error> {
        // Echo the parsed category/paging through the response so tests can see it arrived.
        Ok(LookupPageDto {
            items: vec![self.lookup(filter.category.as_deref().unwrap_or("MALE"), 1)],
            total: 503,
            page: filter.page.unwrap_or(1),
            limit: filter.limit.unwrap_or(25),
        })
    }

    async fn get_lookup_by_id(&self, id: Uuid) -> Result<Option<LookupDto>, Dbpg004Error> {
        Ok((id == self.lookup_id).then(|| self.lookup("MALE", 1)))
    }

    async fn get_summary(&self) -> Result<Dbpg004SummaryDto, Dbpg004Error> {
        Ok(Dbpg004SummaryDto { total_lookups: 1300, active_lookups: 1290, total_categories: 14 })
    }

    async fn create_lookup(
        &self,
        cmd: CreateLookupCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<LookupDto, Dbpg004Error> {
        if cmd.code.trim().is_empty() {
            return Err(Dbpg004Error::Validation("Code is required".to_string()));
        }
        if cmd.code.eq_ignore_ascii_case("MALE") {
            return Err(Dbpg004Error::DuplicateCode("MALE".to_string()));
        }
        Ok(self.lookup(&cmd.code.trim().to_uppercase(), 1))
    }

    async fn update_lookup(
        &self,
        _id: Uuid,
        cmd: UpdateLookupCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<LookupDto, Dbpg004Error> {
        if cmd.row_version != 1 {
            return Err(Dbpg004Error::VersionConflict);
        }
        Ok(self.lookup("MALE", 2))
    }

    async fn delete_lookup(&self, id: Uuid) -> Result<(), Dbpg004Error> {
        // The fixture lookup is still referenced by other records
        if id == self.lookup_id {
            return Err(Dbpg004Error::InUse("still referenced".to_string()));
        }
        Ok(())
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
        assert_eq!(program_code, "DBPG004");
        let allowed = match (username, action) {
            ("admin", _) => true,
            ("writer", PermissionAction::Read | PermissionAction::Write) => true,
            ("reader", PermissionAction::Read) => true,
            _ => false,
        };
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

fn router_as(m: impl Dbpg004Commands + 'static, username: &str) -> Router {
    let state = Dbpg004State { commands: Arc::new(m), access: Arc::new(MockAccess) };
    Router::new()
        .nest(DBPG004_PATH, dbpg004_routes_with_state(state))
        .layer(Extension(user(username)))
}

fn mock() -> MockDbpg004Commands {
    MockDbpg004Commands { lookup_id: Uuid::now_v7() }
}

fn router(mock: MockDbpg004Commands) -> Router {
    router_as(mock, "admin")
}

async fn body_json<T: serde::de::DeserializeOwned>(resp: axum::response::Response) -> T {
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
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

#[tokio::test]
async fn test_list_lookups_parses_category_and_paging() {
    let resp = router(mock())
        .oneshot(get("/db/dbpg004/lookups?category=prefix&page=4&limit=10&active=true"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let page: LookupPageDto = body_json(resp).await;
    assert_eq!((page.page, page.limit, page.total), (4, 10, 503));
    assert_eq!(page.items[0].code, "prefix");
}

#[tokio::test]
async fn test_categories_summary_and_unknown_lookup() {
    let app = router(mock());
    let cats: Vec<LookupCategoryDto> =
        body_json(app.clone().oneshot(get("/db/dbpg004/categories")).await.unwrap()).await;
    assert_eq!(cats[0].category, "gender");

    let summary: Dbpg004SummaryDto =
        body_json(app.clone().oneshot(get("/db/dbpg004/summary")).await.unwrap()).await;
    assert_eq!(summary.total_categories, 14);

    let missing = app
        .oneshot(get(&format!("/db/dbpg004/lookups/{}", Uuid::now_v7())))
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_write_error_status_codes() {
    let m = mock();
    let id = m.lookup_id;
    let app = router(m);
    let create = |code: &str| {
        json_request(
            "POST",
            "/db/dbpg004/lookups",
            serde_json::json!({ "category": "gender", "code": code, "active": true,
                                "name_th": "ชื่อ", "name_en": "" }),
        )
    };

    let ok = app.clone().oneshot(create("x1")).await.unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    let created: LookupDto = body_json(ok).await;
    assert_eq!(created.code, "X1");

    assert_eq!(app.clone().oneshot(create(" ")).await.unwrap().status(), StatusCode::BAD_REQUEST);
    assert_eq!(app.clone().oneshot(create("male")).await.unwrap().status(), StatusCode::CONFLICT);

    let update = |row_version: i64| {
        json_request(
            "PUT",
            &format!("/db/dbpg004/lookups/{id}"),
            serde_json::json!({ "active": true, "name_th": "ชาย", "name_en": "", "row_version": row_version }),
        )
    };
    assert_eq!(app.clone().oneshot(update(1)).await.unwrap().status(), StatusCode::OK);
    assert_eq!(app.oneshot(update(8)).await.unwrap().status(), StatusCode::PRECONDITION_FAILED);
}

#[tokio::test]
async fn test_delete_lookup_status_codes() {
    let m = mock();
    let in_use_id = m.lookup_id;
    let app = router(m);
    let delete = |id: Uuid| {
        Request::builder()
            .uri(format!("/db/dbpg004/lookups/{id}"))
            .method("DELETE")
            .body(Body::empty())
            .unwrap()
    };

    let ok = app.clone().oneshot(delete(Uuid::now_v7())).await.unwrap();
    assert_eq!(ok.status(), StatusCode::NO_CONTENT);

    let conflict = app.oneshot(delete(in_use_id)).await.unwrap();
    assert_eq!(conflict.status(), StatusCode::CONFLICT);
    let body: serde_json::Value = body_json(conflict).await;
    assert_eq!(body["code"], "IN_USE");
}

#[tokio::test]
async fn test_permissions_read_write_approve() {
    let status = |username: &str, req: Request<Body>| {
        let app = router_as(mock(), username);
        async move { app.oneshot(req).await.unwrap().status() }
    };
    let create = || json_request("POST", "/db/dbpg004/lookups", serde_json::json!({ "category": "gender", "code": "x1", "active": true, "name_th": "ชื่อ", "name_en": "", "row_version": 1 }));
    let update = || json_request("PUT", "/db/dbpg004/lookups/00000000-0000-7000-8000-000000000001", serde_json::json!({ "category": "gender", "code": "x1", "active": true, "name_th": "ชื่อ", "name_en": "", "row_version": 1 }));
    let delete = || {
        Request::builder().uri("/db/dbpg004/lookups/00000000-0000-7000-8000-000000000001").method("DELETE").body(Body::empty()).unwrap()
    };

    // a stranger holds nothing
    assert_eq!(status("stranger", get("/db/dbpg004/summary")).await, StatusCode::FORBIDDEN);
    assert_eq!(status("stranger", create()).await, StatusCode::FORBIDDEN);

    // reader: READ only
    assert_eq!(status("reader", get("/db/dbpg004/summary")).await, StatusCode::OK);
    assert_eq!(status("reader", create()).await, StatusCode::FORBIDDEN);
    assert_eq!(status("reader", update()).await, StatusCode::FORBIDDEN);
    assert_eq!(status("reader", delete()).await, StatusCode::FORBIDDEN);

    // writer: READ + WRITE, but deleting needs APPROVE
    assert_eq!(status("writer", create()).await, StatusCode::OK);
    assert_eq!(status("writer", update()).await, StatusCode::OK);
    assert_eq!(status("writer", delete()).await, StatusCode::FORBIDDEN);

    // admin: APPROVE
    assert_eq!(status("admin", delete()).await, StatusCode::NO_CONTENT);
}
