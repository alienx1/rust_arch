//! DBPG000 router: Organizations & Companies list/read endpoints.
//!
//! Mounted at `/api/v1/db/dbpg000` (path = `db_program.path` of program `DBPG000`).

use application_master::features::su::access_control::{AccessControl, PermissionAction};
use application_master::features::db::dbpg000::{
    CompanyFilterQuery, Dbpg000Commands, Dbpg000Error,
};
use axum::{
    Extension, Json, Router,
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use infrastructure::middleware::AuthenticatedUser;
use infrastructure::wire::WireFormat;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use super::access::require_any;
use crate::connection::AppConnection;

/// Path of program `DBPG000` below the API prefix.
pub const DBPG000_PATH: &str = "/db/dbpg000";
/// The list pages at `/db/dbpg000` are reached from the menu entry `DBPG001` (organization data),
/// so READ on either program opens them.
const PROGRAMS: [&str; 2] = ["DBPG000", "DBPG001"];

/// State of the standalone router used by tests (the app router takes both from `AppConnection`)
#[derive(Clone)]
pub struct Dbpg000State {
    pub commands: Arc<dyn Dbpg000Commands>,
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<Dbpg000State> for Arc<dyn Dbpg000Commands> {
    fn from_ref(s: &Dbpg000State) -> Self {
        s.commands.clone()
    }
}

impl FromRef<Dbpg000State> for Arc<dyn AccessControl> {
    fn from_ref(s: &Dbpg000State) -> Self {
        s.access.clone()
    }
}

/// Standardized error response body for organization endpoints.
#[derive(Serialize)]
pub struct Dbpg000ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `Dbpg000Error`.
pub struct Dbpg000ApiError(pub Dbpg000Error);

impl From<Dbpg000Error> for Dbpg000ApiError {
    fn from(err: Dbpg000Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for Dbpg000ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            Dbpg000Error::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            Dbpg000Error::DuplicateCode(msg) => (
                StatusCode::CONFLICT,
                "DUPLICATE_CODE",
                msg.clone(),
            ),
            Dbpg000Error::VersionConflict => (
                StatusCode::PRECONDITION_FAILED,
                "VERSION_CONFLICT",
                "Data has been modified by another transaction. Please reload and try again.".to_string(),
            ),
            Dbpg000Error::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                msg.clone(),
            ),
            Dbpg000Error::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                format!("Database error: {msg}"),
            ),
            Dbpg000Error::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                msg.clone(),
            ),
        };

        let body = Json(Dbpg000ErrorResponse {
            success: false,
            code: code.to_string(),
            message: msg,
        });

        (status, body).into_response()
    }
}

/// List all organizations
pub async fn list_organizations_handler(
    State(commands): State<Arc<dyn Dbpg000Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg000ApiError> {
    if let Err(denied) = require_any(&access, &user, &PROGRAMS, PermissionAction::Read).await {
        return Ok(denied);
    }
    let orgs = commands.list_organizations().await?;
    Ok(wire.respond(orgs))
}
/// Get organization by ID
pub async fn get_organization_handler(
    State(commands): State<Arc<dyn Dbpg000Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg000ApiError> {
    if let Err(denied) = require_any(&access, &user, &PROGRAMS, PermissionAction::Read).await {
        return Ok(denied);
    }
    let org = commands
        .get_organization_by_id(id)
        .await?
        .ok_or_else(|| Dbpg000Error::NotFound(format!("Organization {id} not found")))?;
    Ok(wire.respond(org))
}
/// List companies with filters
pub async fn list_companies_handler(
    State(commands): State<Arc<dyn Dbpg000Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<CompanyFilterQuery>,
) -> Result<Response, Dbpg000ApiError> {
    if let Err(denied) = require_any(&access, &user, &PROGRAMS, PermissionAction::Read).await {
        return Ok(denied);
    }
    let companies = commands.list_companies(filter).await?;
    Ok(wire.respond(companies))
}
/// Get company by ID
pub async fn get_company_handler(
    State(commands): State<Arc<dyn Dbpg000Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg000ApiError> {
    if let Err(denied) = require_any(&access, &user, &PROGRAMS, PermissionAction::Read).await {
        return Ok(denied);
    }
    let company = commands
        .get_company_by_id(id)
        .await?
        .ok_or_else(|| Dbpg000Error::NotFound(format!("Company {id} not found")))?;
    Ok(wire.respond(company))
}
/// List company types
pub async fn list_company_types_handler(
    State(commands): State<Arc<dyn Dbpg000Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg000ApiError> {
    if let Err(denied) = require_any(&access, &user, &PROGRAMS, PermissionAction::Read).await {
        return Ok(denied);
    }
    let types = commands.list_company_types().await?;
    Ok(wire.respond(types))
}
/// Get KPI summary stats
pub async fn get_summary_handler(
    State(commands): State<Arc<dyn Dbpg000Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg000ApiError> {
    if let Err(denied) = require_any(&access, &user, &PROGRAMS, PermissionAction::Read).await {
        return Ok(denied);
    }
    let summary = commands.get_summary().await?;
    Ok(wire.respond(summary))
}
/// Sub-router for DBPG000 endpoints mounted into `AppConnection`.
pub fn dbpg000_routes() -> Router<AppConnection> {
    Router::new()
        .route("/organizations", get(list_organizations_handler))
        .route("/organizations/summary", get(get_summary_handler))
        .route("/organizations/{id}", get(get_organization_handler))
        .route("/companies", get(list_companies_handler))
        .route("/companies/types", get(list_company_types_handler))
        .route("/companies/{id}", get(get_company_handler))
}

/// Same routes with explicit state (useful for tests).
pub fn dbpg000_routes_with_state(state: Dbpg000State) -> Router {
    Router::new()
        .route("/organizations", get(list_organizations_handler))
        .route("/organizations/summary", get(get_summary_handler))
        .route("/organizations/{id}", get(get_organization_handler))
        .route("/companies", get(list_companies_handler))
        .route("/companies/types", get(list_company_types_handler))
        .route("/companies/{id}", get(get_company_handler))
        .with_state(state)
}
