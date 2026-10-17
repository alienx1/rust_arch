//! Integration tests for DBPG000 router endpoints (`/db/dbpg000/...`).
//! Located at `handler/api/master/tests/dbpg000_router_test.rs`.

use api_master::routers::{DBPG000_PATH, Dbpg000State, dbpg000_routes_with_state};
use application_master::features::db::dbpg000::{
    CompanyDto, CompanyFilterQuery, CompanyTypeLookupDto, Dbpg000Commands, Dbpg000Error,
    OrganizationDto, OrganizationSummaryDto,
};
use application_master::features::su::access_control::{
    AccessControl, AccessError, Actor, PermissionAction,
};
use async_trait::async_trait;
use axum::{Extension, Router};
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use infrastructure::middleware::AuthenticatedUser;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

struct MockDbpg000Commands {
    org_id: Uuid,
    comp_id: Uuid,
}

#[async_trait]
impl Dbpg000Commands for MockDbpg000Commands {
    async fn list_organizations(&self) -> Result<Vec<OrganizationDto>, Dbpg000Error> {
        Ok(vec![OrganizationDto {
            id: self.org_id,
            code: "ARCH".to_string(),
            registration_number: "0000000000000".to_string(),
            active: true,
            name_th: "องค์กร ARCH".to_string(),
            name_en: "ARCH Organization".to_string(),
            company_count: 1,
            row_version: 1,
            image_url: None,
        }])
    }

    async fn get_organization_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<OrganizationDto>, Dbpg000Error> {
        if id == self.org_id {
            Ok(Some(OrganizationDto {
                id: self.org_id,
                code: "ARCH".to_string(),
                registration_number: "0000000000000".to_string(),
                active: true,
                name_th: "องค์กร ARCH".to_string(),
                name_en: "ARCH Organization".to_string(),
                company_count: 1,
                row_version: 1,
                image_url: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_companies(
        &self,
        _filter: CompanyFilterQuery,
    ) -> Result<Vec<CompanyDto>, Dbpg000Error> {
        Ok(vec![CompanyDto {
            id: self.comp_id,
            organization_id: self.org_id,
            organization_code: "ARCH".to_string(),
            organization_name: "องค์กร ARCH".to_string(),
            code: "ARCH-HQ".to_string(),
            master_lookup_id: Uuid::now_v7(),
            company_type_name: "บริษัทจำกัด".to_string(),
            active: true,
            name_th: "บริษัท ARCH จำกัด".to_string(),
            name_en: "ARCH Company Limited".to_string(),
            row_version: 1,
            image_url: None,
        }])
    }

    async fn get_company_by_id(&self, id: Uuid) -> Result<Option<CompanyDto>, Dbpg000Error> {
        if id == self.comp_id {
            Ok(Some(CompanyDto {
                id: self.comp_id,
                organization_id: self.org_id,
                organization_code: "ARCH".to_string(),
                organization_name: "องค์กร ARCH".to_string(),
                code: "ARCH-HQ".to_string(),
                master_lookup_id: Uuid::now_v7(),
                company_type_name: "บริษัทจำกัด".to_string(),
                active: true,
                name_th: "บริษัท ARCH จำกัด".to_string(),
                name_en: "ARCH Company Limited".to_string(),
                row_version: 1,
                image_url: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_company_types(&self) -> Result<Vec<CompanyTypeLookupDto>, Dbpg000Error> {
        Ok(vec![CompanyTypeLookupDto {
            id: Uuid::now_v7(),
            code: "PRIVATE".to_string(),
            name_th: "บริษัทจำกัด".to_string(),
            name_en: "Company Limited".to_string(),
        }])
    }

    async fn get_summary(&self) -> Result<OrganizationSummaryDto, Dbpg000Error> {
        Ok(OrganizationSummaryDto {
            total_organizations: 1,
            active_organizations: 1,
            total_companies: 1,
            active_companies: 1,
        })
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
        assert!(program_code == "DBPG000" || program_code == "DBPG001");
        let allowed = match (username, action) {
            ("admin", _) => true,
            ("writer", PermissionAction::Read | PermissionAction::Write) => true,
            ("reader", PermissionAction::Read) => true,
            // Menu entry DBPG001 opens the page at /db/dbpg000: its READ is enough
            ("org_reader", PermissionAction::Read) => program_code == "DBPG001",
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

fn create_router_as(username: &str, org_id: Uuid, comp_id: Uuid) -> Router {
    let commands = Arc::new(MockDbpg000Commands { org_id, comp_id });
    let state = Dbpg000State { commands, access: Arc::new(MockAccess) };
    Router::new()
        .nest(DBPG000_PATH, dbpg000_routes_with_state(state))
        .layer(Extension(user(username)))
}

/// Router acting as `admin` (all rights)
fn create_test_router(org_id: Uuid, comp_id: Uuid) -> Router {
    create_router_as("admin", org_id, comp_id)
}

#[tokio::test]
async fn test_get_organizations_endpoint() {
    let org_id = Uuid::now_v7();
    let comp_id = Uuid::now_v7();
    let app = create_test_router(org_id, comp_id);

    let req = Request::builder()
        .uri("/db/dbpg000/organizations")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let orgs: Vec<OrganizationDto> = serde_json::from_slice(&body).unwrap();
    assert_eq!(orgs.len(), 1);
    assert_eq!(orgs[0].code, "ARCH");
}

#[tokio::test]
async fn test_get_companies_endpoint() {
    let org_id = Uuid::now_v7();
    let comp_id = Uuid::now_v7();
    let app = create_test_router(org_id, comp_id);

    let req = Request::builder()
        .uri("/db/dbpg000/companies")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let comps: Vec<CompanyDto> = serde_json::from_slice(&body).unwrap();
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].code, "ARCH-HQ");
}

#[tokio::test]
async fn test_get_summary_endpoint() {
    let org_id = Uuid::now_v7();
    let comp_id = Uuid::now_v7();
    let app = create_test_router(org_id, comp_id);

    let req = Request::builder()
        .uri("/db/dbpg000/organizations/summary")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: OrganizationSummaryDto = serde_json::from_slice(&body).unwrap();
    assert_eq!(summary.total_organizations, 1);
    assert_eq!(summary.total_companies, 1);
}

#[tokio::test]
async fn test_legacy_organization_paths_are_gone() {
    let app = create_test_router(Uuid::now_v7(), Uuid::now_v7());
    let req = Request::builder().uri("/organizations").body(Body::empty()).unwrap();
    assert_eq!(app.oneshot(req).await.unwrap().status(), StatusCode::NOT_FOUND);
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).method("GET").body(Body::empty()).unwrap()
}

#[tokio::test]
async fn test_every_get_requires_read() {
    let id = Uuid::now_v7();
    let uris = [
        "/db/dbpg000/organizations".to_string(),
        "/db/dbpg000/organizations/summary".to_string(),
        format!("/db/dbpg000/organizations/{id}"),
        "/db/dbpg000/companies".to_string(),
        "/db/dbpg000/companies/types".to_string(),
        format!("/db/dbpg000/companies/{id}"),
    ];
    for uri in &uris {
        let stranger = create_router_as("stranger", id, id);
        let resp = stranger.oneshot(get_request(uri)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN, "{uri}");
        let reader = create_router_as("reader", id, id);
        let resp = reader.oneshot(get_request(uri)).await.unwrap();
        assert_ne!(resp.status(), StatusCode::FORBIDDEN, "{uri}");
    }
}

#[tokio::test]
async fn test_read_on_dbpg001_alone_opens_the_organization_list() {
    let id = Uuid::now_v7();
    let resp = create_router_as("org_reader", id, id)
        .oneshot(get_request("/db/dbpg000/organizations"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
