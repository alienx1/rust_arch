//! Integration tests for DBPG001 router endpoints (`/db/dbpg001/...`).
//! Located at `handler/api/master/tests/dbpg001_router_test.rs`.

use api_master::routers::{DBPG001_PATH, Dbpg001State, dbpg001_routes_with_state};
use application_master::features::db::dbpg001::{
    CompanyDto, CompanyTypeLookupDto, CreateCompanyCommand, CreateOrganizationCommand,
    Dbpg001Commands, Dbpg001Error, OrganizationDto, StoredImage, UpdateCompanyCommand,
    UpdateOrganizationCommand,
};
use application_master::features::su::access_control::{
    AccessControl, AccessError, Actor, OwnEmployee, OwnUnit, PermissionAction,
};
use async_trait::async_trait;
use axum::{Extension, Router};
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use infrastructure::middleware::AuthenticatedUser;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

/// First bytes of a PNG: all the mock needs to hand back
const PNG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

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
            image_url: None,
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
                image_url: None,
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
                image_url: None,
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
            image_url: None,
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
            image_url: None,
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
            image_url: None,
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
            image_url: None,
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

    async fn get_organization_image(&self, id: Uuid) -> Result<StoredImage, Dbpg001Error> {
        if id == self.org_id {
            Ok(StoredImage { bytes: PNG.to_vec(), content_type: "image/png" })
        } else {
            Err(Dbpg001Error::NotFound(format!("{id} has no image")))
        }
    }

    async fn set_organization_image(&self, id: Uuid, bytes: Vec<u8>) -> Result<OrganizationDto, Dbpg001Error> {
        if bytes.len() > 10 {
            return Err(Dbpg001Error::Validation("Image must be PNG, JPEG, GIF or WebP".to_string()));
        }
        let mut org = self.get_organization_by_id(id).await?.ok_or_else(|| Dbpg001Error::NotFound(id.to_string()))?;
        org.image_url = Some(format!("/api/v1/db/dbpg001/organizations/{id}/image?v=abc"));
        Ok(org)
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
        assert!(matches!(program_code, "DBPG001" | "DBPG000"));
        let allowed = match (username, action) {
            ("admin", _) => true,
            ("writer", PermissionAction::Read | PermissionAction::Write) => true,
            ("reader", PermissionAction::Read) => true,
            _ => false,
        };
        Ok(allowed.then(|| Actor { user_id: Uuid::nil(), username: username.to_string() }))
    }

    /// `member` has no rights but belongs to the organization `MEMBER_ORG` and the company `MEMBER_COMPANY`
    async fn own_employee(&self, username: &str) -> Result<Option<OwnEmployee>, AccessError> {
        let unit = |id| OwnUnit { id, code: String::new(), name_th: String::new(), name_en: String::new(), image_url: None };
        Ok((username == "member").then(|| OwnEmployee {
            id: Uuid::nil(),
            image_url: None,
            organization: Some(unit(MEMBER_ORG)),
            company: Some(unit(MEMBER_COMPANY)),
            is_hq: false,
            unlimited: false,
            choices: vec![],
            branches: vec![],
        }))
    }
}

const MEMBER_ORG: Uuid = Uuid::from_u128(0xA);
const MEMBER_COMPANY: Uuid = Uuid::from_u128(0xB);

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

#[tokio::test]
async fn test_organization_image_is_served_with_a_safe_content_type() {
    let org_id = Uuid::now_v7();
    let app = create_router_as("reader", org_id, Uuid::now_v7());
    let resp = app
        .oneshot(Request::builder().uri(format!("/db/dbpg001/organizations/{org_id}/image")).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers()["content-type"], "image/png");
    assert_eq!(resp.headers()["x-content-type-options"], "nosniff");
    assert!(resp.headers()["cache-control"].to_str().unwrap().starts_with("private"));
    assert_eq!(&to_bytes(resp.into_body(), usize::MAX).await.unwrap()[..], &PNG[..]);
}

#[tokio::test]
async fn test_organization_without_an_image_is_404() {
    let app = create_router_as("reader", Uuid::now_v7(), Uuid::now_v7());
    let resp = app
        .oneshot(Request::builder().uri(format!("/db/dbpg001/organizations/{}/image", Uuid::now_v7())).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_uploading_an_image_needs_write_permission() {
    let org_id = Uuid::now_v7();
    let put = |bytes: &'static [u8]| {
        Request::builder()
            .uri(format!("/db/dbpg001/organizations/{org_id}/image"))
            .method("PUT")
            .header("content-type", "image/png")
            .body(Body::from(bytes))
            .unwrap()
    };

    let denied = create_router_as("reader", org_id, Uuid::now_v7()).oneshot(put(&PNG)).await.unwrap();
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    let ok = create_router_as("writer", org_id, Uuid::now_v7()).oneshot(put(&PNG)).await.unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_slice(&to_bytes(ok.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert!(body["image_url"].as_str().unwrap().contains("/image?v="));

    let bad = create_router_as("writer", org_id, Uuid::now_v7())
        .oneshot(put(b"<svg onload=alert(1)></svg>"))
        .await
        .unwrap();
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_everybody_sees_the_logo_of_their_own_organization_and_company_only() {
    // `member` has no right at all
    let get = |app: Router, uri: String| async move {
        app.oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap()).await.unwrap().status()
    };
    // own organization: allowed
    let own = get(create_router_as("member", MEMBER_ORG, Uuid::now_v7()), format!("/db/dbpg001/organizations/{MEMBER_ORG}/image")).await;
    assert_eq!(own, StatusCode::OK);

    // somebody else's organization: forbidden
    let other = get(create_router_as("member", Uuid::now_v7(), Uuid::now_v7()), format!("/db/dbpg001/organizations/{}/image", Uuid::now_v7())).await;
    assert_eq!(other, StatusCode::FORBIDDEN);

    // the company id is not an organization id
    let mixed = get(create_router_as("member", MEMBER_ORG, Uuid::now_v7()), format!("/db/dbpg001/organizations/{MEMBER_COMPANY}/image")).await;
    assert_eq!(mixed, StatusCode::FORBIDDEN);
}
