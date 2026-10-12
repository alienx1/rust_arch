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
    extract::{FromRef, Path, State},
    http::StatusCode,
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
/// Sub-router for DBPG001 endpoints mounted into `AppConnection`.
pub fn dbpg001_routes() -> Router<AppConnection> {
    Router::new()
        .route("/organizations", get(list_organizations_handler).post(create_organization_handler))
        .route("/organizations/{id}", get(get_organization_handler).put(update_organization_handler).delete(delete_organization_handler))
        .route("/companies", axum::routing::post(create_company_handler))
        .route("/companies/types", get(list_company_types_handler))
        .route("/companies/{id}", get(get_company_handler).put(update_company_handler).delete(delete_company_handler))
}

/// Same routes with explicit state (useful for tests).
pub fn dbpg001_routes_with_state(state: Dbpg001State) -> Router {
    Router::new()
        .route("/organizations", get(list_organizations_handler).post(create_organization_handler))
        .route("/organizations/{id}", get(get_organization_handler).put(update_organization_handler).delete(delete_organization_handler))
        .route("/companies", axum::routing::post(create_company_handler))
        .route("/companies/types", get(list_company_types_handler))
        .route("/companies/{id}", get(get_company_handler).put(update_company_handler).delete(delete_company_handler))
        .with_state(state)
}
