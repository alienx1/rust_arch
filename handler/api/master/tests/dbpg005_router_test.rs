//! Integration tests for DBPG005 router endpoints (`/db/dbpg005/...`).
//! Located at `handler/api/master/tests/dbpg005_router_test.rs`.

use api_master::routers::{DBPG005_PATH, Dbpg005State, dbpg005_routes_with_state};
use application_master::features::db::dbpg005::{
    CreateDistrictCommand, CreatePostalCodeCommand, CreateProvinceCommand,
    CreateSubDistrictCommand, Dbpg005Commands, Dbpg005Error, Dbpg005SummaryDto, GeoFilterQuery,
    GeoNodeDto, GeoPageDto, PostalCodeDto, PostalFilterQuery, PostalPageDto, UpdateGeoNodeCommand,
    UpdatePostalCodeCommand,
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

struct MockDbpg005Commands {
    id: Uuid,
}

impl MockDbpg005Commands {
    fn node(&self, level: &str, code: &str, row_version: i64) -> GeoNodeDto {
        GeoNodeDto {
            id: self.id,
            level: level.to_string(),
            code: code.to_string(),
            active: true,
            name_th: "กรุงเทพมหานคร".to_string(),
            name_en: "Bangkok".to_string(),
            province_id: None,
            province_name: String::new(),
            district_id: None,
            district_name: String::new(),
            postal_codes: vec![],
            row_version,
        }
    }

    fn postal(&self, code: &str, row_version: i64) -> PostalCodeDto {
        PostalCodeDto {
            id: self.id,
            postal_code: code.to_string(),
            active: true,
            province_id: Uuid::now_v7(),
            province_name: "กรุงเทพมหานคร".to_string(),
            district_id: None,
            district_name: String::new(),
            sub_district_id: None,
            sub_district_name: String::new(),
            row_version,
        }
    }

    /// The mock's own id is deletable; any other id is still referenced
    fn delete_outcome(&self, id: Uuid) -> Result<(), Dbpg005Error> {
        if id == self.id {
            Ok(())
        } else {
            Err(Dbpg005Error::InUse("still referenced".to_string()))
        }
    }

    /// Echoes the parsed filter through the page so tests can see it arrived.
    fn page(&self, level: &str, filter: &GeoFilterQuery) -> GeoPageDto {
        GeoPageDto {
            items: vec![self.node(level, filter.search.as_deref().unwrap_or("10"), 1)],
            total: 8939,
            page: filter.page.unwrap_or(1),
            limit: filter.limit.unwrap_or(25),
        }
    }
}

#[async_trait]
impl Dbpg005Commands for MockDbpg005Commands {
    async fn list_provinces(&self, f: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error> {
        Ok(self.page("province", &f))
    }
    async fn list_districts(&self, f: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error> {
        Ok(self.page("district", &f))
    }
    async fn list_sub_districts(&self, f: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error> {
        Ok(self.page("sub_district", &f))
    }
    async fn list_postal_codes(&self, f: PostalFilterQuery) -> Result<PostalPageDto, Dbpg005Error> {
        Ok(PostalPageDto {
            items: vec![self.postal(f.search.as_deref().unwrap_or("10110"), 1)],
            total: 7418,
            page: f.page.unwrap_or(1),
            limit: f.limit.unwrap_or(25),
        })
    }

    async fn get_province(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error> {
        Ok((id == self.id).then(|| self.node("province", "10", 1)))
    }
    async fn get_district(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error> {
        Ok((id == self.id).then(|| self.node("district", "1001", 1)))
    }
    async fn get_sub_district(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error> {
        Ok((id == self.id).then(|| self.node("sub_district", "100101", 1)))
    }
    async fn get_postal_code(&self, id: Uuid) -> Result<Option<PostalCodeDto>, Dbpg005Error> {
        Ok((id == self.id).then(|| self.postal("10110", 1)))
    }

    async fn get_summary(&self) -> Result<Dbpg005SummaryDto, Dbpg005Error> {
        Ok(Dbpg005SummaryDto { provinces: 77, districts: 1008, sub_districts: 8939, postal_codes: 7418 })
    }

    async fn create_province(&self, cmd: CreateProvinceCommand, _a: Option<Uuid>) -> Result<GeoNodeDto, Dbpg005Error> {
        if cmd.code.trim().is_empty() {
            return Err(Dbpg005Error::Validation("Code is required".to_string()));
        }
        if cmd.code == "10" {
            return Err(Dbpg005Error::DuplicateCode("10".to_string()));
        }
        Ok(self.node("province", cmd.code.trim(), 1))
    }
    async fn create_district(&self, cmd: CreateDistrictCommand, _a: Option<Uuid>) -> Result<GeoNodeDto, Dbpg005Error> {
        Ok(self.node("district", cmd.code.trim(), 1))
    }
    async fn create_sub_district(&self, cmd: CreateSubDistrictCommand, _a: Option<Uuid>) -> Result<GeoNodeDto, Dbpg005Error> {
        Ok(self.node("sub_district", cmd.code.trim(), 1))
    }

    async fn update_province(&self, _id: Uuid, cmd: UpdateGeoNodeCommand, _a: Option<Uuid>) -> Result<GeoNodeDto, Dbpg005Error> {
        if cmd.row_version != 1 {
            return Err(Dbpg005Error::VersionConflict);
        }
        Ok(self.node("province", "10", 2))
    }
    async fn update_district(&self, _id: Uuid, _cmd: UpdateGeoNodeCommand, _a: Option<Uuid>) -> Result<GeoNodeDto, Dbpg005Error> {
        Ok(self.node("district", "1001", 2))
    }
    async fn update_sub_district(&self, _id: Uuid, _cmd: UpdateGeoNodeCommand, _a: Option<Uuid>) -> Result<GeoNodeDto, Dbpg005Error> {
        Ok(self.node("sub_district", "100101", 2))
    }

    async fn create_postal_code(&self, cmd: CreatePostalCodeCommand, _a: Option<Uuid>) -> Result<PostalCodeDto, Dbpg005Error> {
        Ok(self.postal(cmd.postal_code.trim(), 1))
    }
    async fn update_postal_code(&self, _id: Uuid, cmd: UpdatePostalCodeCommand, _a: Option<Uuid>) -> Result<PostalCodeDto, Dbpg005Error> {
        Ok(self.postal(cmd.postal_code.trim(), 2))
    }

    async fn delete_province(&self, id: Uuid) -> Result<(), Dbpg005Error> {
        self.delete_outcome(id)
    }
    async fn delete_district(&self, id: Uuid) -> Result<(), Dbpg005Error> {
        self.delete_outcome(id)
    }
    async fn delete_sub_district(&self, id: Uuid) -> Result<(), Dbpg005Error> {
        self.delete_outcome(id)
    }
    async fn delete_postal_code(&self, id: Uuid) -> Result<(), Dbpg005Error> {
        self.delete_outcome(id)
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
        assert_eq!(program_code, "DBPG005");
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

fn router_as(m: impl Dbpg005Commands + 'static, username: &str) -> Router {
    let state = Dbpg005State { commands: Arc::new(m), access: Arc::new(MockAccess) };
    Router::new()
        .nest(DBPG005_PATH, dbpg005_routes_with_state(state))
        .layer(Extension(user(username)))
}

fn mock() -> MockDbpg005Commands {
    MockDbpg005Commands { id: Uuid::now_v7() }
}

fn router(m: MockDbpg005Commands) -> Router {
    router_as(m, "admin")
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
async fn test_each_level_has_its_own_list_route_with_filters_parsed() {
    let app = router(mock());
    for (path, level) in [
        ("provinces", "province"),
        ("districts", "district"),
        ("sub-districts", "sub_district"),
    ] {
        let uri = format!(
            "/db/dbpg005/{path}?province_id={}&district_id={}&search=x1&page=3&limit=50&active=true",
            Uuid::now_v7(),
            Uuid::now_v7()
        );
        let resp = app.clone().oneshot(get(&uri)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "{path}");
        let page: GeoPageDto = body_json(resp).await;
        assert_eq!(page.items[0].level, level);
        assert_eq!(page.items[0].code, "x1");
        assert_eq!((page.page, page.limit), (3, 50));
    }
}

#[tokio::test]
async fn test_postal_codes_summary_and_unknown_unit() {
    let app = router(mock());
    let resp = app
        .clone()
        .oneshot(get(&format!(
            "/db/dbpg005/postal-codes?sub_district_id={}&search=10200",
            Uuid::now_v7()
        )))
        .await
        .unwrap();
    let page: PostalPageDto = body_json(resp).await;
    assert_eq!(page.items[0].postal_code, "10200");
    assert_eq!(page.total, 7418);

    let summary: Dbpg005SummaryDto =
        body_json(app.clone().oneshot(get("/db/dbpg005/summary")).await.unwrap()).await;
    assert_eq!(summary.sub_districts, 8939);

    for path in ["provinces", "districts", "sub-districts", "postal-codes"] {
        let missing = app
            .clone()
            .oneshot(get(&format!("/db/dbpg005/{path}/{}", Uuid::now_v7())))
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test]
async fn test_write_error_status_codes() {
    let m = mock();
    let id = m.id;
    let app = router(m);
    let create = |code: &str| {
        json_request(
            "POST",
            "/db/dbpg005/provinces",
            serde_json::json!({ "code": code, "active": true, "name_th": "ใหม่", "name_en": "" }),
        )
    };

    assert_eq!(app.clone().oneshot(create("99")).await.unwrap().status(), StatusCode::OK);
    assert_eq!(app.clone().oneshot(create(" ")).await.unwrap().status(), StatusCode::BAD_REQUEST);
    assert_eq!(app.clone().oneshot(create("10")).await.unwrap().status(), StatusCode::CONFLICT);

    let update = |row_version: i64| {
        json_request(
            "PUT",
            &format!("/db/dbpg005/provinces/{id}"),
            serde_json::json!({ "active": true, "name_th": "กรุงเทพมหานคร", "name_en": "", "row_version": row_version }),
        )
    };
    assert_eq!(app.clone().oneshot(update(1)).await.unwrap().status(), StatusCode::OK);
    assert_eq!(app.oneshot(update(4)).await.unwrap().status(), StatusCode::PRECONDITION_FAILED);
}

#[tokio::test]
async fn test_delete_returns_204_or_409_in_use_for_every_level() {
    let m = mock();
    let id = m.id;
    let app = router(m);
    let delete = |path: &str, id: Uuid| {
        Request::builder()
            .uri(format!("/db/dbpg005/{path}/{id}"))
            .method("DELETE")
            .body(Body::empty())
            .unwrap()
    };
    for path in ["provinces", "districts", "sub-districts", "postal-codes"] {
        let ok = app.clone().oneshot(delete(path, id)).await.unwrap();
        assert_eq!(ok.status(), StatusCode::NO_CONTENT, "{path}");

        let in_use = app.clone().oneshot(delete(path, Uuid::now_v7())).await.unwrap();
        assert_eq!(in_use.status(), StatusCode::CONFLICT, "{path}");
        let body: serde_json::Value = body_json(in_use).await;
        assert_eq!(body["code"], "IN_USE", "{path}");
    }
}

#[tokio::test]
async fn test_permissions_read_write_approve() {
    let status = |username: &str, req: Request<Body>| {
        let app = router_as(MockDbpg005Commands { id: Uuid::from_u128(1) }, username);
        async move { app.oneshot(req).await.unwrap().status() }
    };
    let create = || json_request("POST", "/db/dbpg005/provinces", serde_json::json!({ "code": "99", "active": true, "name_th": "ใหม่", "name_en": "", "row_version": 1 }));
    let update = || json_request("PUT", "/db/dbpg005/provinces/00000000-0000-0000-0000-000000000001", serde_json::json!({ "code": "99", "active": true, "name_th": "ใหม่", "name_en": "", "row_version": 1 }));
    let delete = || {
        Request::builder().uri("/db/dbpg005/provinces/00000000-0000-0000-0000-000000000001").method("DELETE").body(Body::empty()).unwrap()
    };

    // a stranger holds nothing
    assert_eq!(status("stranger", get("/db/dbpg005/summary")).await, StatusCode::FORBIDDEN);
    assert_eq!(status("stranger", create()).await, StatusCode::FORBIDDEN);

    // reader: READ only
    assert_eq!(status("reader", get("/db/dbpg005/summary")).await, StatusCode::OK);
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
