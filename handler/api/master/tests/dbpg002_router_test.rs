//! Integration tests for DBPG002 router endpoints (`/db/dbpg002/...`).
//! Located at `handler/api/master/tests/dbpg002_router_test.rs`.

use api_master::routers::{DBPG002_PATH, Dbpg002State, dbpg002_routes_with_state};
use application_master::features::db::dbpg002::{
    BranchDto, BranchFilterQuery, BranchTypeLookupDto, CompanyOptionDto, CreateBranchCommand, PermissionGroupOptionDto,
    CreateDepartmentCommand, Dbpg002Commands, Dbpg002Error, Dbpg002SummaryDto, DepartmentDto,
    DepartmentFilterQuery, UpdateBranchCommand, UpdateDepartmentCommand,
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

struct MockDbpg002Commands {
    company_id: Uuid,
    branch_id: Uuid,
    department_id: Uuid,
}

impl MockDbpg002Commands {
    fn branch(&self) -> BranchDto {
        BranchDto {
            id: self.branch_id,
            organization_id: Uuid::now_v7(),
            company_id: self.company_id,
            company_code: "CMP01".to_string(),
            company_name: "บริษัท 1".to_string(),
            code: "HO".to_string(),
            master_lookup_id: Uuid::now_v7(),
            branch_type_code: "HEAD".to_string(),
            branch_type_name: "สำนักงานใหญ่".to_string(),
            is_headquarter: true,
            active: true,
            name_th: "สำนักงานใหญ่".to_string(),
            name_en: "Head Office".to_string(),
            department_count: 1,
            row_version: 1,
        }
    }

    fn department(&self, code: &str, row_version: i64) -> DepartmentDto {
        let b = self.branch();
        DepartmentDto {
            id: self.department_id,
            organization_id: b.organization_id,
            company_id: b.company_id,
            company_code: b.company_code,
            company_name: b.company_name,
            branch_id: b.id,
            branch_code: b.code,
            branch_name: b.name_th,
            code: code.to_string(),
            active: true,
            name_th: "ไอที".to_string(),
            name_en: "IT".to_string(),
            default_permission_group_id: None,
            default_permission_group_code: None,
            row_version,
        }
    }
}

#[async_trait]
impl Dbpg002Commands for MockDbpg002Commands {
    async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg002Error> {
        Ok(vec![CompanyOptionDto {
            id: self.company_id,
            organization_id: Uuid::now_v7(),
            code: "CMP01".to_string(),
            name_th: "บริษัท 1".to_string(),
            name_en: "Company 1".to_string(),
        }])
    }

    async fn list_permission_group_options(&self) -> Result<Vec<PermissionGroupOptionDto>, Dbpg002Error> {
        Ok(vec![PermissionGroupOptionDto { id: uuid::Uuid::nil(), code: "STAFF".to_string() }])
    }

    async fn list_branch_types(&self) -> Result<Vec<BranchTypeLookupDto>, Dbpg002Error> {
        Ok(vec![])
    }

    async fn list_branches(
        &self,
        _filter: BranchFilterQuery,
    ) -> Result<Vec<BranchDto>, Dbpg002Error> {
        Ok(vec![self.branch()])
    }

    async fn get_branch_by_id(&self, id: Uuid) -> Result<Option<BranchDto>, Dbpg002Error> {
        Ok((id == self.branch_id).then(|| self.branch()))
    }

    async fn list_departments(
        &self,
        filter: DepartmentFilterQuery,
    ) -> Result<Vec<DepartmentDto>, Dbpg002Error> {
        // Echo the parsed branch filter through the code so the test can see it arrived.
        let code = if filter.branch_id == Some(self.branch_id) { "IT" } else { "OTHER" };
        Ok(vec![self.department(code, 1)])
    }

    async fn get_department_by_id(&self, id: Uuid) -> Result<Option<DepartmentDto>, Dbpg002Error> {
        Ok((id == self.department_id).then(|| self.department("IT", 1)))
    }

    async fn get_summary(&self) -> Result<Dbpg002SummaryDto, Dbpg002Error> {
        Ok(Dbpg002SummaryDto {
            total_branches: 1,
            active_branches: 1,
            total_departments: 1,
            active_departments: 1,
        })
    }

    async fn create_branch(
        &self,
        _cmd: CreateBranchCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<BranchDto, Dbpg002Error> {
        Ok(self.branch())
    }

    async fn update_branch(
        &self,
        _id: Uuid,
        _cmd: UpdateBranchCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<BranchDto, Dbpg002Error> {
        Ok(self.branch())
    }

    async fn create_department(
        &self,
        cmd: CreateDepartmentCommand,
        _actor_id: Option<Uuid>,
        may_assign_groups: bool,
    ) -> Result<DepartmentDto, Dbpg002Error> {
        // What the real executor does: a default permission group needs the right to assign groups
        if cmd.default_permission_group_id.is_some() && !may_assign_groups {
            return Err(Dbpg002Error::NotAllowed("needs the right to assign permission groups".to_string()));
        }
        if cmd.code.trim().is_empty() {
            return Err(Dbpg002Error::Validation("Department code is required".to_string()));
        }
        Ok(self.department(&cmd.code.trim().to_uppercase(), 1))
    }

    async fn update_department(
        &self,
        _id: Uuid,
        cmd: UpdateDepartmentCommand,
        _actor_id: Option<Uuid>,
        may_assign_groups: bool,
    ) -> Result<DepartmentDto, Dbpg002Error> {
        // The mock department has no default, so any default in the request is a change
        if cmd.default_permission_group_id.is_some() && !may_assign_groups {
            return Err(Dbpg002Error::NotAllowed("needs the right to assign permission groups".to_string()));
        }
        if cmd.row_version != 1 {
            return Err(Dbpg002Error::VersionConflict);
        }
        Ok(self.department(&cmd.code.trim().to_uppercase(), 2))
    }

    async fn delete_branch(&self, id: Uuid) -> Result<(), Dbpg002Error> {
        if id == self.branch_id {
            Err(Dbpg002Error::InUse("still referenced".to_string()))
        } else {
            Ok(())
        }
    }

    async fn delete_department(&self, id: Uuid) -> Result<(), Dbpg002Error> {
        if id == self.department_id {
            Ok(())
        } else {
            Err(Dbpg002Error::NotFound(id.to_string()))
        }
    }
}

fn mock() -> MockDbpg002Commands {
    MockDbpg002Commands {
        company_id: Uuid::now_v7(),
        branch_id: Uuid::now_v7(),
        department_id: Uuid::now_v7(),
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
        // Only the admin may also assign permission groups (SUPG004): the writer manages departments only
        assert!(matches!(program_code, "DBPG002" | "SUPG004"), "{program_code}");
        let allowed = match (program_code, username, action) {
            (_, "admin", _) => true,
            ("DBPG002", "writer", PermissionAction::Read | PermissionAction::Write) => true,
            ("DBPG002", "reader", PermissionAction::Read) => true,
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

fn router_as(mock: MockDbpg002Commands, username: &str) -> Router {
    let state = Dbpg002State { commands: Arc::new(mock), access: Arc::new(MockAccess) };
    Router::new()
        .nest(DBPG002_PATH, dbpg002_routes_with_state(state))
        .layer(Extension(user(username)))
}

fn router(mock: MockDbpg002Commands) -> Router {
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
async fn test_get_branches_and_summary_endpoints() {
    let app = router(mock());

    let resp = app.clone().oneshot(get("/db/dbpg002/branches")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let branches: Vec<BranchDto> = body_json(resp).await;
    assert_eq!(branches.len(), 1);
    assert!(branches[0].is_headquarter);

    let resp = app.oneshot(get("/db/dbpg002/summary")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let summary: Dbpg002SummaryDto = body_json(resp).await;
    assert_eq!(summary.total_departments, 1);
}

#[tokio::test]
async fn test_departments_query_filter_reaches_handler() {
    let m = mock();
    let branch_id = m.branch_id;
    let app = router(m);

    let resp = app
        .oneshot(get(&format!("/db/dbpg002/departments?branch_id={branch_id}&active=true")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let departments: Vec<DepartmentDto> = body_json(resp).await;
    assert_eq!(departments[0].code, "IT");
}

#[tokio::test]
async fn test_unknown_branch_is_404_with_error_body() {
    let app = router(mock());
    let resp = app
        .oneshot(get(&format!("/db/dbpg002/branches/{}", Uuid::now_v7())))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let err: serde_json::Value = body_json(resp).await;
    assert_eq!(err["success"], false);
    assert_eq!(err["code"], "NOT_FOUND");
}

#[tokio::test]
async fn test_create_department_maps_validation_error_to_400() {
    let m = mock();
    let branch_id = m.branch_id;
    let app = router(m);
    let body = |code: &str| {
        serde_json::json!({
            "branch_id": branch_id, "code": code, "active": true,
            "name_th": "ไอที", "name_en": "IT"
        })
    };

    let ok = app
        .clone()
        .oneshot(json_request("POST", "/db/dbpg002/departments", body("it")))
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    let created: DepartmentDto = body_json(ok).await;
    assert_eq!(created.code, "IT");

    let bad = app
        .oneshot(json_request("POST", "/db/dbpg002/departments", body("  ")))
        .await
        .unwrap();
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_department_version_conflict_is_412() {
    let m = mock();
    let (id, branch_id) = (m.department_id, m.branch_id);
    let app = router(m);
    let body = |row_version: i64| {
        serde_json::json!({
            "branch_id": branch_id, "code": "it", "active": true,
            "name_th": "ไอที", "name_en": "IT", "row_version": row_version
        })
    };

    let ok = app
        .clone()
        .oneshot(json_request("PUT", &format!("/db/dbpg002/departments/{id}"), body(1)))
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    let updated: DepartmentDto = body_json(ok).await;
    assert_eq!(updated.row_version, 2);

    let stale = app
        .oneshot(json_request("PUT", &format!("/db/dbpg002/departments/{id}"), body(7)))
        .await
        .unwrap();
    assert_eq!(stale.status(), StatusCode::PRECONDITION_FAILED);
}

/// The web client serializes ids with `serialize_id`: real UUIDs (16 bytes in CBOR). Plain CBOR
/// text ids are NOT accepted for the command's `Uuid` fields, which is why `serialize_id` exists.
#[tokio::test]
async fn test_create_department_accepts_uuid_ids_over_cbor() {
    use infrastructure::wire::{APPLICATION_CBOR, to_cbor_vec};

    #[derive(serde::Serialize)]
    struct WebCreateDepartmentInput<Id> {
        branch_id: Id,
        code: String,
        active: bool,
        name_th: String,
        name_en: String,
    }
    fn input<Id>(branch_id: Id) -> WebCreateDepartmentInput<Id> {
        WebCreateDepartmentInput {
            branch_id,
            code: "acc".to_string(),
            active: true,
            name_th: "บัญชี".to_string(),
            name_en: "Accounting".to_string(),
        }
    }
    let post = |payload: Vec<u8>| {
        Request::builder()
            .uri("/db/dbpg002/departments")
            .method("POST")
            .header("content-type", APPLICATION_CBOR)
            .header("accept", "application/json")
            .body(Body::from(payload))
            .unwrap()
    };

    let m = mock();
    let branch_id = m.branch_id;
    let app = router(m);

    let resp = app
        .clone()
        .oneshot(post(to_cbor_vec(&input(branch_id)).unwrap()))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let created: DepartmentDto = body_json(resp).await;
    assert_eq!(created.code, "ACC");

    let as_text = app
        .oneshot(post(to_cbor_vec(&input(branch_id.to_string())).unwrap()))
        .await
        .unwrap();
    assert_eq!(as_text.status(), StatusCode::BAD_REQUEST);
}

fn delete(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).method("DELETE").body(Body::empty()).unwrap()
}

#[tokio::test]
async fn test_delete_department_is_204_and_unknown_is_404() {
    let m = mock();
    let id = m.department_id;
    let app = router(m);

    let ok = app
        .clone()
        .oneshot(delete(&format!("/db/dbpg002/departments/{id}")))
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::NO_CONTENT);

    let missing = app
        .oneshot(delete(&format!("/db/dbpg002/departments/{}", Uuid::now_v7())))
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_branch_in_use_is_409_in_use() {
    let m = mock();
    let (in_use, free) = (m.branch_id, Uuid::now_v7());
    let app = router(m);

    let ok = app
        .clone()
        .oneshot(delete(&format!("/db/dbpg002/branches/{free}")))
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::NO_CONTENT);

    let resp = app
        .oneshot(delete(&format!("/db/dbpg002/branches/{in_use}")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let err: serde_json::Value = body_json(resp).await;
    assert_eq!(err["code"], "IN_USE");
}

fn branch_body(row_version: Option<i64>) -> serde_json::Value {
    let mut v = serde_json::json!({
        "company_id": Uuid::now_v7(), "code": "b1", "master_lookup_id": Uuid::now_v7(), "active": true,
        "name_th": "สาขา", "name_en": "Branch"
    });
    if let Some(rv) = row_version {
        v["row_version"] = rv.into();
    }
    v
}

fn department_body(row_version: Option<i64>) -> serde_json::Value {
    let mut v = serde_json::json!({
        "branch_id": Uuid::now_v7(), "code": "it", "active": true,
        "name_th": "ไอที", "name_en": "IT"
    });
    if let Some(rv) = row_version {
        v["row_version"] = rv.into();
    }
    v
}

#[tokio::test]
async fn test_stranger_is_forbidden_on_every_get() {
    let id = Uuid::now_v7();
    for uri in [
        "/db/dbpg002/companies".to_string(),
        "/db/dbpg002/summary".to_string(),
        "/db/dbpg002/branches/types".to_string(),
        "/db/dbpg002/branches".to_string(),
        format!("/db/dbpg002/branches/{id}"),
        "/db/dbpg002/departments".to_string(),
        format!("/db/dbpg002/departments/{id}"),
    ] {
        let resp = router_as(mock(), "stranger").oneshot(get(&uri)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN, "{uri}");
    }
}

#[tokio::test]
async fn test_reader_can_get_but_not_write_or_delete() {
    let m = mock();
    let (branch, dept) = (m.branch_id, m.department_id);
    let app = router_as(m, "reader");
    assert_eq!(app.clone().oneshot(get("/db/dbpg002/branches")).await.unwrap().status(), StatusCode::OK);
    assert_eq!(app.clone().oneshot(get("/db/dbpg002/summary")).await.unwrap().status(), StatusCode::OK);

    let denied = [
        json_request("POST", "/db/dbpg002/branches", branch_body(None)),
        json_request("PUT", &format!("/db/dbpg002/branches/{branch}"), branch_body(Some(1))),
        json_request("POST", "/db/dbpg002/departments", department_body(None)),
        json_request("PUT", &format!("/db/dbpg002/departments/{dept}"), department_body(Some(1))),
        delete(&format!("/db/dbpg002/branches/{branch}")),
        delete(&format!("/db/dbpg002/departments/{dept}")),
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
    let (branch, dept) = (m.branch_id, m.department_id);
    let app = router_as(m, "writer");

    let created = app
        .clone()
        .oneshot(json_request("POST", "/db/dbpg002/departments", department_body(None)))
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::OK);
    let updated = app
        .clone()
        .oneshot(json_request("PUT", &format!("/db/dbpg002/departments/{dept}"), department_body(Some(1))))
        .await
        .unwrap();
    assert_eq!(updated.status(), StatusCode::OK);

    for uri in [format!("/db/dbpg002/branches/{branch}"), format!("/db/dbpg002/departments/{dept}")] {
        let resp = app.clone().oneshot(delete(&uri)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN, "{uri}");
    }
}

#[tokio::test]
async fn test_admin_can_delete() {
    let m = mock();
    let (branch, dept) = (Uuid::now_v7(), m.department_id);
    let app = router_as(m, "admin");
    for uri in [format!("/db/dbpg002/branches/{branch}"), format!("/db/dbpg002/departments/{dept}")] {
        let resp = app.clone().oneshot(delete(&uri)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT, "{uri}");
    }
}

#[tokio::test]
async fn test_permission_group_options_are_for_those_who_may_assign_groups() {
    let resp = router_as(mock(), "admin").oneshot(get("/db/dbpg002/options/permission-groups")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list: serde_json::Value = body_json(resp).await;
    assert_eq!(list[0]["code"], "STAFF");
    // A department manager who cannot assign groups does not get the list of groups
    for user in ["writer", "reader"] {
        let resp = router_as(mock(), user).oneshot(get("/db/dbpg002/options/permission-groups")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN, "{user}");
    }
}

fn department_body_with_default(default_group: Option<uuid::Uuid>, row_version: i64) -> serde_json::Value {
    serde_json::json!({
        "branch_id": uuid::Uuid::nil(), "code": "it", "active": true, "name_th": "ไอที", "name_en": "IT",
        "default_permission_group_id": default_group, "row_version": row_version
    })
}

#[tokio::test]
async fn test_setting_a_default_permission_group_needs_the_right_to_assign_groups() {
    let group = Some(uuid::Uuid::now_v7());
    let create = |user: &'static str, group| {
        let app = router_as(mock(), user);
        async move {
            app.oneshot(json_request("POST", "/db/dbpg002/departments", department_body_with_default(group, 1))).await.unwrap().status()
        }
    };
    // A department manager cannot hand out a permission group (it would reach every account of the
    // department, themselves included); whoever may assign groups on SUPG004 can
    assert_eq!(create("writer", group).await, StatusCode::FORBIDDEN);
    assert_eq!(create("admin", group).await, StatusCode::OK);
    assert_eq!(create("writer", None).await, StatusCode::OK, "a department without a default is still a DBPG002 matter");

    let update = |user: &'static str, group| {
        let app = router_as(mock(), user);
        async move {
            let id = mock().department_id;
            app.oneshot(json_request("PUT", &format!("/db/dbpg002/departments/{id}"), department_body_with_default(group, 1)))
                .await
                .unwrap()
                .status()
        }
    };
    assert_eq!(update("writer", group).await, StatusCode::FORBIDDEN, "changing the default is granting it");
    assert_eq!(update("admin", group).await, StatusCode::OK);
    assert_eq!(update("writer", None).await, StatusCode::OK, "clearing it, or not touching it, grants nothing");
}
