//! Integration tests for DBPG003 router endpoints (`/db/dbpg003/...`).
//! Located at `handler/api/master/tests/dbpg003_router_test.rs`.

use api_master::routers::{DBPG003_PATH, Dbpg003State, dbpg003_routes_with_state};
use application_master::features::db::dbpg003::{
    BranchOptionDto, CompanyOptionDto, CreateEmployeeCommand, Dbpg003Commands, Dbpg003Error,
    Dbpg003SummaryDto, DepartmentOptionDto, EmployeeDto, EmployeeFilterQuery, EmployeePageDto,
    PrefixOptionDto, UpdateEmployeeCommand,
};
use application_master::features::su::access_control::{AccessControl, AccessError, Actor, PermissionAction};
use async_trait::async_trait;
use axum::{Extension, Router};
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use infrastructure::middleware::AuthenticatedUser;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

struct MockDbpg003Commands {
    employee_id: Uuid,
    department_id: Uuid,
}

impl MockDbpg003Commands {
    fn employee(&self, code: &str, row_version: i64) -> EmployeeDto {
        EmployeeDto {
            id: self.employee_id,
            organization_id: Uuid::now_v7(),
            company_id: Uuid::now_v7(),
            company_code: "CMP01".to_string(),
            company_name: "บริษัท 1".to_string(),
            branch_id: Uuid::now_v7(),
            branch_code: "HO".to_string(),
            branch_name: "สำนักงานใหญ่".to_string(),
            department_id: self.department_id,
            department_code: "IT".to_string(),
            department_name: "ไอที".to_string(),
            person_name_id: Uuid::now_v7(),
            prefix_id: Uuid::now_v7(),
            prefix_name: "นาย".to_string(),
            code: code.to_string(),
            full_name: "นาย สมชาย ใจดี".to_string(),
            first_name_th: "สมชาย".to_string(),
            middle_name_th: String::new(),
            last_name_th: "ใจดี".to_string(),
            nickname_th: String::new(),
            first_name_en: "Somchai".to_string(),
            middle_name_en: String::new(),
            last_name_en: "Jaidee".to_string(),
            nickname_en: String::new(),
            active: true,
            has_user_account: true,
            row_version,
        }
    }
}

#[async_trait]
impl Dbpg003Commands for MockDbpg003Commands {
    async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg003Error> {
        Ok(vec![])
    }
    async fn list_branch_options(&self) -> Result<Vec<BranchOptionDto>, Dbpg003Error> {
        Ok(vec![])
    }
    async fn list_department_options(&self) -> Result<Vec<DepartmentOptionDto>, Dbpg003Error> {
        Ok(vec![])
    }
    async fn list_prefixes(&self) -> Result<Vec<PrefixOptionDto>, Dbpg003Error> {
        Ok(vec![PrefixOptionDto {
            id: Uuid::now_v7(),
            code: "001".to_string(),
            name_th: "นาย".to_string(),
            name_en: "Mr.".to_string(),
        }])
    }

    async fn list_employees(
        &self,
        filter: EmployeeFilterQuery,
    ) -> Result<EmployeePageDto, Dbpg003Error> {
        // Echo the parsed paging/search through the response so tests can see it arrived.
        Ok(EmployeePageDto {
            items: vec![self.employee(filter.search.as_deref().unwrap_or("E001"), 1)],
            total: 41,
            page: filter.page.unwrap_or(1),
            limit: filter.limit.unwrap_or(25),
        })
    }

    async fn get_employee_by_id(&self, id: Uuid) -> Result<Option<EmployeeDto>, Dbpg003Error> {
        Ok((id == self.employee_id).then(|| self.employee("E001", 1)))
    }

    async fn get_summary(&self) -> Result<Dbpg003SummaryDto, Dbpg003Error> {
        Ok(Dbpg003SummaryDto { total_employees: 41, active_employees: 40, with_user_account: 3 })
    }

    async fn create_employee(
        &self,
        cmd: CreateEmployeeCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<EmployeeDto, Dbpg003Error> {
        if cmd.code.trim().is_empty() {
            return Err(Dbpg003Error::Validation("Employee code is required".to_string()));
        }
        Ok(self.employee(&cmd.code.trim().to_uppercase(), 1))
    }

    async fn update_employee(
        &self,
        _id: Uuid,
        cmd: UpdateEmployeeCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<EmployeeDto, Dbpg003Error> {
        if cmd.row_version != 1 {
            return Err(Dbpg003Error::VersionConflict);
        }
        Ok(self.employee(&cmd.code.trim().to_uppercase(), 2))
    }

    async fn delete_employee(&self, id: Uuid) -> Result<(), Dbpg003Error> {
        // The fixture employee has a user account, so it is still referenced
        if id == self.employee_id {
            return Err(Dbpg003Error::InUse("still referenced by a user account".to_string()));
        }
        Ok(())
    }
}

fn mock() -> MockDbpg003Commands {
    MockDbpg003Commands { employee_id: Uuid::now_v7(), department_id: Uuid::now_v7() }
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
        assert_eq!(program_code, "DBPG003");
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

fn router_as(mock: MockDbpg003Commands, username: &str) -> Router {
    let state = Dbpg003State { commands: Arc::new(mock), access: Arc::new(MockAccess) };
    Router::new()
        .nest(DBPG003_PATH, dbpg003_routes_with_state(state))
        .layer(Extension(user(username)))
}

fn router(mock: MockDbpg003Commands) -> Router {
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

fn employee_body(department_id: Uuid, code: &str, row_version: Option<i64>) -> serde_json::Value {
    let mut v = serde_json::json!({
        "department_id": department_id, "prefix_id": Uuid::now_v7(), "code": code, "active": true,
        "first_name_th": "สมชาย", "middle_name_th": "", "last_name_th": "ใจดี", "nickname_th": "",
        "first_name_en": "", "middle_name_en": "", "last_name_en": "", "nickname_en": ""
    });
    if let Some(rv) = row_version {
        v["row_version"] = rv.into();
    }
    v
}

#[tokio::test]
async fn test_list_employees_parses_paging_and_search() {
    let app = router(mock());
    let resp = app
        .oneshot(get("/db/dbpg003/employees?page=3&limit=10&search=E777&active=true"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let page: EmployeePageDto = body_json(resp).await;
    assert_eq!((page.page, page.limit, page.total), (3, 10, 41));
    assert_eq!(page.items[0].code, "E777");
}

#[tokio::test]
async fn test_summary_and_prefixes_endpoints() {
    let app = router(mock());
    let summary: Dbpg003SummaryDto =
        body_json(app.clone().oneshot(get("/db/dbpg003/summary")).await.unwrap()).await;
    assert_eq!(summary.with_user_account, 3);

    let prefixes: Vec<PrefixOptionDto> =
        body_json(app.oneshot(get("/db/dbpg003/prefixes")).await.unwrap()).await;
    assert_eq!(prefixes[0].name_th, "นาย");
}

#[tokio::test]
async fn test_unknown_employee_is_404() {
    let app = router(mock());
    let resp = app
        .oneshot(get(&format!("/db/dbpg003/employees/{}", Uuid::now_v7())))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let err: serde_json::Value = body_json(resp).await;
    assert_eq!(err["code"], "NOT_FOUND");
}

#[tokio::test]
async fn test_create_validation_and_update_conflict_status_codes() {
    let m = mock();
    let (dept, id) = (m.department_id, m.employee_id);
    let app = router(m);

    let ok = app
        .clone()
        .oneshot(json_request("POST", "/db/dbpg003/employees", employee_body(dept, "e9", None)))
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    let created: EmployeeDto = body_json(ok).await;
    assert_eq!(created.code, "E9");

    let bad = app
        .clone()
        .oneshot(json_request("POST", "/db/dbpg003/employees", employee_body(dept, " ", None)))
        .await
        .unwrap();
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);

    let uri = format!("/db/dbpg003/employees/{id}");
    let updated = app
        .clone()
        .oneshot(json_request("PUT", &uri, employee_body(dept, "e9", Some(1))))
        .await
        .unwrap();
    assert_eq!(updated.status(), StatusCode::OK);
    let updated: EmployeeDto = body_json(updated).await;
    assert_eq!(updated.row_version, 2);

    let stale = app
        .oneshot(json_request("PUT", &uri, employee_body(dept, "e9", Some(9))))
        .await
        .unwrap();
    assert_eq!(stale.status(), StatusCode::PRECONDITION_FAILED);
}

#[tokio::test]
async fn test_delete_employee_status_codes() {
    let m = mock();
    let in_use_id = m.employee_id;
    let app = router(m);
    let delete = |id: Uuid| {
        Request::builder()
            .uri(format!("/db/dbpg003/employees/{id}"))
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

fn delete(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).method("DELETE").body(Body::empty()).unwrap()
}

#[tokio::test]
async fn test_stranger_is_forbidden_on_every_get() {
    let id = Uuid::now_v7();
    for uri in [
        "/db/dbpg003/companies".to_string(),
        "/db/dbpg003/branches".to_string(),
        "/db/dbpg003/departments".to_string(),
        "/db/dbpg003/prefixes".to_string(),
        "/db/dbpg003/summary".to_string(),
        "/db/dbpg003/employees".to_string(),
        format!("/db/dbpg003/employees/{id}"),
    ] {
        let resp = router_as(mock(), "stranger").oneshot(get(&uri)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN, "{uri}");
    }
}

#[tokio::test]
async fn test_reader_can_get_but_not_write_or_delete() {
    let m = mock();
    let (dept, id) = (m.department_id, m.employee_id);
    let app = router_as(m, "reader");
    assert_eq!(app.clone().oneshot(get("/db/dbpg003/employees")).await.unwrap().status(), StatusCode::OK);

    let denied = [
        json_request("POST", "/db/dbpg003/employees", employee_body(dept, "e1", None)),
        json_request("PUT", &format!("/db/dbpg003/employees/{id}"), employee_body(dept, "e1", Some(1))),
        delete(&format!("/db/dbpg003/employees/{id}")),
    ];
    for req in denied {
        let uri = req.uri().to_string();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN, "{uri}");
    }
}

#[tokio::test]
async fn test_writer_can_post_put_but_not_delete() {
    let m = mock();
    let (dept, id) = (m.department_id, m.employee_id);
    let app = router_as(m, "writer");

    let created = app
        .clone()
        .oneshot(json_request("POST", "/db/dbpg003/employees", employee_body(dept, "e1", None)))
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::OK);
    let updated = app
        .clone()
        .oneshot(json_request("PUT", &format!("/db/dbpg003/employees/{id}"), employee_body(dept, "e1", Some(1))))
        .await
        .unwrap();
    assert_eq!(updated.status(), StatusCode::OK);

    let resp = app.oneshot(delete(&format!("/db/dbpg003/employees/{id}"))).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_can_delete() {
    let resp = router_as(mock(), "admin")
        .oneshot(delete(&format!("/db/dbpg003/employees/{}", Uuid::now_v7())))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}
