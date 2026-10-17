//! DBPG001 router: Organization & Company create/update/delete (detail forms).
//!
//! Mounted at `/api/v1/db/dbpg001` (path = `db_program.path` of program `DBPG001`).
//! Sub-screens: `/organizations` (DBPG001 ORG) and `/companies` (DBPG001 COMPANY).

use application_master::features::su::access_control::{AccessControl, PermissionAction};
use application_master::features::db::dbpg001::{
    CreateCompanyCommand, CreateOrganizationCommand, Dbpg001Commands, Dbpg001Error,
    UpdateCompanyCommand, UpdateOrganizationCommand,
};
use axum::{
    Extension, Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, FromRef, Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use infrastructure::middleware::AuthenticatedUser;
use infrastructure::wire::{WireBody, WireFormat};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use super::access::require;
use crate::connection::AppConnection;

/// Path of program `DBPG001` below the API prefix.
pub const DBPG001_PATH: &str = "/db/dbpg001";
const PROGRAM: &str = "DBPG001";

/// State of the standalone router used by tests (the app router takes both from `AppConnection`)
#[derive(Clone)]
pub struct Dbpg001State {
    pub commands: Arc<dyn Dbpg001Commands>,
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<Dbpg001State> for Arc<dyn Dbpg001Commands> {
    fn from_ref(s: &Dbpg001State) -> Self {
        s.commands.clone()
    }
}

impl FromRef<Dbpg001State> for Arc<dyn AccessControl> {
    fn from_ref(s: &Dbpg001State) -> Self {
        s.access.clone()
    }
}

/// Standardized error response body for organization endpoints.
#[derive(Serialize)]
pub struct Dbpg001ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `Dbpg001Error`.
pub struct Dbpg001ApiError(pub Dbpg001Error);

impl From<Dbpg001Error> for Dbpg001ApiError {
    fn from(err: Dbpg001Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for Dbpg001ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            Dbpg001Error::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            Dbpg001Error::DuplicateCode(msg) => (
                StatusCode::CONFLICT,
                "DUPLICATE_CODE",
                msg.clone(),
            ),
            Dbpg001Error::VersionConflict => (
                StatusCode::PRECONDITION_FAILED,
                "VERSION_CONFLICT",
                "Data has been modified by another transaction. Please reload and try again.".to_string(),
            ),
            Dbpg001Error::InUse(msg) => (StatusCode::CONFLICT, "IN_USE", msg.clone()),
            Dbpg001Error::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                msg.clone(),
            ),
            Dbpg001Error::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                format!("Database error: {msg}"),
            ),
            Dbpg001Error::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                msg.clone(),
            ),
        };

        let body = Json(Dbpg001ErrorResponse {
            success: false,
            code: code.to_string(),
            message: msg,
        });

        (status, body).into_response()
    }
}

/// List all organizations
pub async fn list_organizations_handler(
    State(commands): State<Arc<dyn Dbpg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let orgs = commands.list_organizations().await?;
    Ok(wire.respond(orgs))
}
/// Get organization by ID
pub async fn get_organization_handler(
    State(commands): State<Arc<dyn Dbpg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let org = commands
        .get_organization_by_id(id)
        .await?
        .ok_or_else(|| Dbpg001Error::NotFound(format!("Organization {id} not found")))?;
    Ok(wire.respond(org))
}
/// Get company by ID
pub async fn get_company_handler(
    State(commands): State<Arc<dyn Dbpg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let company = commands
        .get_company_by_id(id)
        .await?
        .ok_or_else(|| Dbpg001Error::NotFound(format!("Company {id} not found")))?;
    Ok(wire.respond(company))
}
/// List company types
pub async fn list_company_types_handler(
    State(commands): State<Arc<dyn Dbpg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let types = commands.list_company_types().await?;
    Ok(wire.respond(types))
}
/// Create organization
pub async fn create_organization_handler(
    State(commands): State<Arc<dyn Dbpg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<CreateOrganizationCommand>,
) -> Result<Response, Dbpg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    let org = commands.create_organization(cmd, None).await?;
    Ok(wire.respond(org))
}
/// Update organization
pub async fn update_organization_handler(
    State(commands): State<Arc<dyn Dbpg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
    WireBody(cmd): WireBody<UpdateOrganizationCommand>,
) -> Result<Response, Dbpg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    let org = commands.update_organization(id, cmd, None).await?;
    Ok(wire.respond(org))
}
/// Create company
pub async fn create_company_handler(
    State(commands): State<Arc<dyn Dbpg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<CreateCompanyCommand>,
) -> Result<Response, Dbpg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    let company = commands.create_company(cmd, None).await?;
    Ok(wire.respond(company))
}
/// Update company
pub async fn update_company_handler(
    State(commands): State<Arc<dyn Dbpg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
    WireBody(cmd): WireBody<UpdateCompanyCommand>,
) -> Result<Response, Dbpg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    let company = commands.update_company(id, cmd, None).await?;
    Ok(wire.respond(company))
}
/// Delete organization (only when nothing references it)
pub async fn delete_organization_handler(
    State(commands): State<Arc<dyn Dbpg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Approve).await {
        return Ok(denied);
    }
    commands.delete_organization(id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
/// Delete company (only when nothing references it)
pub async fn delete_company_handler(
    State(commands): State<Arc<dyn Dbpg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Approve).await {
        return Ok(denied);
    }
    commands.delete_company(id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
/// Largest request body of an image upload: a little above the 2 MB limit so the feature answers
/// "too large" with a clear message instead of the generic body-limit error.
const IMAGE_BODY_LIMIT: usize = 3 * 1024 * 1024;

/// Programs whose users may see a logo: the detail forms (DBPG001) and the list screen (DBPG000)
/// ... and everybody sees the logo of their own organization / company (the sidebar brand)
async fn may_view_image(
    access: &Arc<dyn AccessControl>,
    user: &AuthenticatedUser,
    id: Uuid,
    company: bool,
) -> Result<(), Response> {
    let denied = match require(access, user, PROGRAM, PermissionAction::Read).await {
        Ok(_) => return Ok(()),
        Err(_) => match require(access, user, "DBPG000", PermissionAction::Read).await {
            Ok(_) => return Ok(()),
            Err(denied) => denied,
        },
    };
    let own = match super::username_of(user) {
        Some(name) => access.own_employee(&name).await.ok().flatten().is_some_and(|e| {
            let unit = if company { e.company } else { e.organization };
            unit.is_some_and(|u| u.id == id)
        }),
        None => false,
    };
    if own { Ok(()) } else { Err(denied) }
}

/// An image as the browser should receive it: the stored type, `nosniff`, private cache (the URL
/// changes with every upload)
fn image_response(image: application_master::features::db::dbpg001::StoredImage) -> Response {
    let mut res = (StatusCode::OK, image.bytes).into_response();
    let h = res.headers_mut();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static(image.content_type));
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("private, max-age=86400"));
    h.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    res
}

macro_rules! image_handlers {
    ($get:ident, $put:ident, $delete:ident, $get_cmd:ident, $set_cmd:ident, $remove_cmd:ident, $company:expr) => {
        /// Logo bytes
        pub async fn $get(
            State(commands): State<Arc<dyn Dbpg001Commands>>,
            State(access): State<Arc<dyn AccessControl>>,
            Extension(user): Extension<AuthenticatedUser>,
            Path(id): Path<Uuid>,
        ) -> Result<Response, Dbpg001ApiError> {
            if let Err(denied) = may_view_image(&access, &user, id, $company).await {
                return Ok(denied);
            }
            Ok(image_response(commands.$get_cmd(id).await?))
        }

        /// Replace the logo: the request body is the raw image file
        pub async fn $put(
            State(commands): State<Arc<dyn Dbpg001Commands>>,
            State(access): State<Arc<dyn AccessControl>>,
            Extension(user): Extension<AuthenticatedUser>,
            wire: WireFormat,
            Path(id): Path<Uuid>,
            body: Bytes,
        ) -> Result<Response, Dbpg001ApiError> {
            if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
                return Ok(denied);
            }
            Ok(wire.respond(commands.$set_cmd(id, body.to_vec()).await?))
        }

        /// Remove the logo
        pub async fn $delete(
            State(commands): State<Arc<dyn Dbpg001Commands>>,
            State(access): State<Arc<dyn AccessControl>>,
            Extension(user): Extension<AuthenticatedUser>,
            wire: WireFormat,
            Path(id): Path<Uuid>,
        ) -> Result<Response, Dbpg001ApiError> {
            if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
                return Ok(denied);
            }
            Ok(wire.respond(commands.$remove_cmd(id).await?))
        }
    };
}

image_handlers!(
    get_organization_image_handler,
    put_organization_image_handler,
    delete_organization_image_handler,
    get_organization_image,
    set_organization_image,
    remove_organization_image,
    false
);
image_handlers!(
    get_company_image_handler,
    put_company_image_handler,
    delete_company_image_handler,
    get_company_image,
    set_company_image,
    remove_company_image,
    true
);

/// Sub-router for DBPG001 endpoints mounted into `AppConnection`.
pub fn dbpg001_routes() -> Router<AppConnection> {
    Router::new()
        .merge(super::contacts_router::dbpg001_contact_routes())
        .route("/organizations", get(list_organizations_handler).post(create_organization_handler))
        .route("/organizations/{id}", get(get_organization_handler).put(update_organization_handler).delete(delete_organization_handler))
        .route("/companies", axum::routing::post(create_company_handler))
        .route("/companies/types", get(list_company_types_handler))
        .route("/companies/{id}", get(get_company_handler).put(update_company_handler).delete(delete_company_handler))
        .route(
            "/organizations/{id}/image",
            get(get_organization_image_handler).put(put_organization_image_handler).delete(delete_organization_image_handler),
        )
        .route(
            "/companies/{id}/image",
            get(get_company_image_handler).put(put_company_image_handler).delete(delete_company_image_handler),
        )
        .layer(DefaultBodyLimit::max(IMAGE_BODY_LIMIT))
}

/// Same routes with explicit state (useful for tests).
pub fn dbpg001_routes_with_state(state: Dbpg001State) -> Router {
    Router::new()
        .route("/organizations", get(list_organizations_handler).post(create_organization_handler))
        .route("/organizations/{id}", get(get_organization_handler).put(update_organization_handler).delete(delete_organization_handler))
        .route("/companies", axum::routing::post(create_company_handler))
        .route("/companies/types", get(list_company_types_handler))
        .route("/companies/{id}", get(get_company_handler).put(update_company_handler).delete(delete_company_handler))
        .route(
            "/organizations/{id}/image",
            get(get_organization_image_handler).put(put_organization_image_handler).delete(delete_organization_image_handler),
        )
        .route(
            "/companies/{id}/image",
            get(get_company_image_handler).put(put_company_image_handler).delete(delete_company_image_handler),
        )
        .layer(DefaultBodyLimit::max(IMAGE_BODY_LIMIT))
        .with_state(state)
}
