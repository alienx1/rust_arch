//! Integration tests for DBPG001 router endpoints (`/db/dbpg001/...`).
//! Located at `handler/api/master/tests/dbpg001_router_test.rs`.

use api_master::routers::{DBPG001_PATH, Dbpg001State, dbpg001_routes_with_state};
use application_master::features::db::dbpg001::{
    CompanyDto, CompanyTypeLookupDto, CreateCompanyCommand, CreateOrganizationCommand,
    Dbpg001Commands, Dbpg001Error, OrganizationDto, UpdateCompanyCommand,
    UpdateOrganizationCommand,
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

/// Id the mock reports as still referenced
const IN_USE_ID: Uuid = Uuid::from_u128(0x1);

struct MockDbpg001Commands {
    org_id: Uuid,
    comp_id: Uuid,
    in_use_id: Uuid,
}

#[async_trait]
impl Dbpg001Commands for MockDbpg001Commands {
    async fn list_organizations(&self) -> Result<Vec<OrganizationDto>, Dbpg001Error> {
        Ok(vec![OrganizationDto {
            id: self.org_id,
            code: "ARCH".to_string(),
            registration_number: "0000000000000".to_string(),
            active: true,
            name_th: "องค์กร ARCH".to_string(),
            name_en: "ARCH Organization".to_string(),
            company_count: 1,
            row_version: 1,
        }])
    }

    async fn get_organization_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<OrganizationDto>, Dbpg001Error> {
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
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_company_by_id(&self, id: Uuid) -> Result<Option<CompanyDto>, Dbpg001Error> {
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
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_company_types(&self) -> Result<Vec<CompanyTypeLookupDto>, Dbpg001Error> {
        Ok(vec![CompanyTypeLookupDto {
            id: Uuid::now_v7(),
            code: "PRIVATE".to_string(),
            name_th: "บริษัทจำกัด".to_string(),
            name_en: "Company Limited".to_string(),
        }])
    }

    async fn create_organization(
        &self,
        cmd: CreateOrganizationCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<OrganizationDto, Dbpg001Error> {
        Ok(OrganizationDto {
            id: Uuid::now_v7(),
            code: cmd.code.to_uppercase(),
            registration_number: cmd.registration_number,
            active: cmd.active,
            name_th: cmd.name_th,
            name_en: cmd.name_en,
            company_count: 0,
            row_version: 1,
        })
    }

    async fn update_organization(
        &self,
        id: Uuid,
        cmd: UpdateOrganizationCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<OrganizationDto, Dbpg001Error> {
        Ok(OrganizationDto {
            id,
            code: cmd.code.to_uppercase(),
            registration_number: cmd.registration_number,
            active: cmd.active,
            name_th: cmd.name_th,
            name_en: cmd.name_en,
            company_count: 0,
            row_version: cmd.row_version + 1,
        })
    }

    async fn create_company(
        &self,
        cmd: CreateCompanyCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<CompanyDto, Dbpg001Error> {
        Ok(CompanyDto {
            id: Uuid::now_v7(),
            organization_id: cmd.organization_id,
            organization_code: "ARCH".to_string(),
            organization_name: "องค์กร ARCH".to_string(),
            code: cmd.code.to_uppercase(),
            master_lookup_id: cmd.master_lookup_id,
            company_type_name: "บริษัทจำกัด".to_string(),
            active: cmd.active,
            name_th: cmd.name_th,
            name_en: cmd.name_en,
            row_version: 1,
        })
    }

    async fn update_company(
        &self,
        id: Uuid,
        cmd: UpdateCompanyCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<CompanyDto, Dbpg001Error> {
        Ok(CompanyDto {
            id,
            organization_id: cmd.organization_id,
            organization_code: "ARCH".to_string(),
            organization_name: "องค์กร ARCH".to_string(),
            code: cmd.code.to_uppercase(),
            master_lookup_id: cmd.master_lookup_id,
            company_type_name: "บริษัทจำกัด".to_string(),
            active: cmd.active,
            name_th: cmd.name_th,
            name_en: cmd.name_en,
            row_version: cmd.row_version + 1,
        })
    }

    async fn delete_organization(&self, id: Uuid) -> Result<(), Dbpg001Error> {
        if id == self.org_id {
            Ok(())
        } else if id == self.in_use_id {
            Err(Dbpg001Error::InUse("still referenced".to_string()))
        } else {
            Err(Dbpg001Error::NotFound(id.to_string()))
        }
    }

    async fn delete_company(&self, id: Uuid) -> Result<(), Dbpg001Error> {
        if id == self.comp_id {
            Ok(())
        } else if id == self.in_use_id {
            Err(Dbpg001Error::InUse("still referenced".to_string()))
        } else {
            Err(Dbpg001Error::NotFound(id.to_string()))
        }
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
        assert_eq!(program_code, "DBPG001");
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

fn create_router_as(username: &str, org_id: Uuid, comp_id: Uuid) -> Router {
    let commands = Arc::new(MockDbpg001Commands { org_id, comp_id, in_use_id: IN_USE_ID });
    let state = Dbpg001State { commands, access: Arc::new(MockAccess) };
    Router::new()
        .nest(DBPG001_PATH, dbpg001_routes_with_state(state))
        .layer(Extension(user(username)))
}

/// Router acting as `admin` (all rights)
fn create_test_router(org_id: Uuid, comp_id: Uuid) -> Router {
    create_router_as("admin", org_id, comp_id)
}

fn sample_create_org() -> CreateOrganizationCommand {
    CreateOrganizationCommand {
        code: "acme".to_string(),
        registration_number: "REG-1".to_string(),
        active: true,
        name_th: "เอซีเอ็มอี".to_string(),
        name_en: "Acme".to_string(),
    }
}

#[tokio::test]
async fn test_create_organization_cbor_binary_request_and_response() {
    use infrastructure::wire::{APPLICATION_CBOR, from_cbor_slice, to_cbor_vec};

    let app = create_test_router(Uuid::now_v7(), Uuid::now_v7());
    let req = Request::builder()
        .uri("/db/dbpg001/organizations")
        .method("POST")
        .header("content-type", APPLICATION_CBOR)
        .header("accept", APPLICATION_CBOR)
        .body(Body::from(to_cbor_vec(&sample_create_org()).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers()["content-type"], APPLICATION_CBOR);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    // A CBOR map header byte (0xa0..=0xbf) proves the body is raw binary, not hex text.
    assert!((0xa0..=0xbf).contains(&body[0]), "response must be raw binary CBOR");
    let org: OrganizationDto = from_cbor_slice(&body).unwrap();
    assert_eq!(org.code, "ACME");
}

#[tokio::test]
async fn test_create_organization_cbor_hex_request_and_response() {
    use infrastructure::wire::{CBOR_HEX, CBOR_HEX_CONTENT_TYPE, from_cbor_slice, to_cbor_hex};

    let app = create_test_router(Uuid::now_v7(), Uuid::now_v7());
    let req = Request::builder()
        .uri("/db/dbpg001/organizations")
        .method("POST")
        .header("content-type", CBOR_HEX)
        .header("accept", CBOR_HEX)
        .body(Body::from(to_cbor_hex(&sample_create_org()).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers()["content-type"], CBOR_HEX_CONTENT_TYPE);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert!(body.iter().all(|b| b.is_ascii_hexdigit()), "response must be lowercase hex text");
    let org: OrganizationDto = from_cbor_slice(&body).unwrap();
    assert_eq!(org.code, "ACME");
}

#[tokio::test]
async fn test_error_response_follows_negotiated_cbor_format() {
    use infrastructure::wire::{
        APPLICATION_CBOR, CBOR_HEX, CBOR_HEX_CONTENT_TYPE, cbor_error_responses, from_cbor_slice,
    };

    let app = create_test_router(Uuid::now_v7(), Uuid::now_v7())
        .layer(axum::middleware::from_fn(cbor_error_responses));
    let bad_request = |media: &str| {
        Request::builder()
            .uri("/db/dbpg001/organizations")
            .method("POST")
            .header("content-type", media)
            .header("accept", media)
            .body(Body::from("zz-not-cbor"))
            .unwrap()
    };

    // Malformed payload => extractor rejection (plain text) comes back as binary CBOR...
    let resp = app.clone().oneshot(bad_request(APPLICATION_CBOR)).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(resp.headers()["content-type"], APPLICATION_CBOR);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert!((0xa0..=0xbf).contains(&body[0]), "error must be raw binary CBOR");
    let err: serde_json::Value = from_cbor_slice(&body).unwrap();
    assert_eq!(err["success"], false);

    // ...or as hex CBOR when hex was negotiated.
    let resp = app.clone().oneshot(bad_request(CBOR_HEX)).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(resp.headers()["content-type"], CBOR_HEX_CONTENT_TYPE);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert!(body.iter().all(|b| b.is_ascii_hexdigit()), "error must be lowercase hex text");
    let err: serde_json::Value = from_cbor_slice(&body).unwrap();
    assert_eq!(err["success"], false);

    // Without CBOR negotiation the error stays untouched.
    let req = Request::builder()
        .uri("/db/dbpg001/organizations")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from("{"))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let content_type = resp.headers().get("content-type").map(|v| v.to_str().unwrap().to_string());
    assert!(!content_type.unwrap_or_default().contains("cbor"));
}

fn delete_request(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).method("DELETE").body(Body::empty()).unwrap()
}

#[tokio::test]
async fn test_delete_organization_and_company_return_no_content() {
    let (org_id, comp_id) = (Uuid::now_v7(), Uuid::now_v7());
    let app = create_test_router(org_id, comp_id);

    let resp = app.clone().oneshot(delete_request(&format!("/db/dbpg001/organizations/{org_id}"))).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    let resp = app.oneshot(delete_request(&format!("/db/dbpg001/companies/{comp_id}"))).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_in_use_returns_conflict_in_use() {
    let app = create_test_router(Uuid::now_v7(), Uuid::now_v7());
    for kind in ["organizations", "companies"] {
        let resp = app.clone().oneshot(delete_request(&format!("/db/dbpg001/{kind}/{IN_USE_ID}"))).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["code"], "IN_USE");
    }
}

#[tokio::test]
async fn test_delete_missing_returns_not_found() {
    let app = create_test_router(Uuid::now_v7(), Uuid::now_v7());
    let resp = app.oneshot(delete_request(&format!("/db/dbpg001/companies/{}", Uuid::now_v7()))).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

fn get_request(uri: &str) -> Request<Body> {
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

fn create_org_request() -> Request<Body> {
    json_request("POST", "/db/dbpg001/organizations", serde_json::to_value(sample_create_org()).unwrap())
}

fn update_org_request(id: Uuid) -> Request<Body> {
    json_request(
        "PUT",
        &format!("/db/dbpg001/organizations/{id}"),
        serde_json::json!({
            "code": "acme", "registration_number": "REG-1", "active": true,
            "name_th": "เอซีเอ็มอี", "name_en": "Acme", "row_version": 1
        }),
    )
}

#[tokio::test]
async fn test_every_get_requires_read() {
    let id = Uuid::now_v7();
    let uris = [
        "/db/dbpg001/organizations".to_string(),
        format!("/db/dbpg001/organizations/{id}"),
        "/db/dbpg001/companies/types".to_string(),
        format!("/db/dbpg001/companies/{id}"),
    ];
    for uri in &uris {
        let resp = create_router_as("stranger", id, id).oneshot(get_request(uri)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN, "{uri}");
        let resp = create_router_as("reader", id, id).oneshot(get_request(uri)).await.unwrap();
        assert_ne!(resp.status(), StatusCode::FORBIDDEN, "{uri}");
    }
}

#[tokio::test]
async fn test_reader_cannot_create_or_update() {
    let id = Uuid::now_v7();
    let app = create_router_as("reader", id, id);
    let resp = app.clone().oneshot(create_org_request()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    let resp = app.oneshot(update_org_request(id)).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_writer_can_create_and_update_but_not_delete() {
    let id = Uuid::now_v7();
    let app = create_router_as("writer", id, id);
    let resp = app.clone().oneshot(create_org_request()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let resp = app.clone().oneshot(update_org_request(id)).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    for kind in ["organizations", "companies"] {
        let resp = app.clone().oneshot(delete_request(&format!("/db/dbpg001/{kind}/{id}"))).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN, "{kind}");
    }
}

#[tokio::test]
async fn test_admin_can_delete() {
    let id = Uuid::now_v7();
    let app = create_router_as("admin", id, id);
    let resp = app.oneshot(delete_request(&format!("/db/dbpg001/organizations/{id}"))).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}
