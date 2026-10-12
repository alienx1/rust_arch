//! DBPG005 router: Geography (provinces, districts, sub-districts, postal codes).
//!
//! Mounted at `/api/v1/db/dbpg005` (path = `db_program.path` of program `DBPG005`).

use application_master::features::db::dbpg005::{
    CreateDistrictCommand, CreatePostalCodeCommand, CreateProvinceCommand,
    CreateSubDistrictCommand, Dbpg005Commands, Dbpg005Error, GeoFilterQuery, PostalFilterQuery,
    UpdateGeoNodeCommand, UpdatePostalCodeCommand,
};
use application_master::features::su::access_control::{AccessControl, PermissionAction};
use axum::{
    Extension, Json, Router,
    extract::{FromRef, Path, Query, State},
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

/// Path of program `DBPG005` below the API prefix.
pub const DBPG005_PATH: &str = "/db/dbpg005";
const PROGRAM: &str = "DBPG005";

/// State of the standalone router used by tests (the app router takes both from `AppConnection`)
#[derive(Clone)]
pub struct Dbpg005State {
    pub commands: Arc<dyn Dbpg005Commands>,
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<Dbpg005State> for Arc<dyn Dbpg005Commands> {
    fn from_ref(s: &Dbpg005State) -> Self {
        s.commands.clone()
    }
}

impl FromRef<Dbpg005State> for Arc<dyn AccessControl> {
    fn from_ref(s: &Dbpg005State) -> Self {
        s.access.clone()
    }
}

/// Standardized error response body for DBPG005 endpoints.
#[derive(Serialize)]
pub struct Dbpg005ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `Dbpg005Error`.
pub struct Dbpg005ApiError(pub Dbpg005Error);

impl From<Dbpg005Error> for Dbpg005ApiError {
    fn from(err: Dbpg005Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for Dbpg005ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            Dbpg005Error::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            Dbpg005Error::DuplicateCode(msg) => {
                (StatusCode::CONFLICT, "DUPLICATE_CODE", msg.clone())
            }
            Dbpg005Error::InUse(msg) => (StatusCode::CONFLICT, "IN_USE", msg.clone()),
            Dbpg005Error::VersionConflict => (
                StatusCode::PRECONDITION_FAILED,
                "VERSION_CONFLICT",
                "Data has been modified by another transaction. Please reload and try again."
                    .to_string(),
            ),
            Dbpg005Error::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone())
            }
            Dbpg005Error::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                format!("Database error: {msg}"),
            ),
            Dbpg005Error::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone())
            }
        };

        let body = Json(Dbpg005ErrorResponse {
            success: false,
            code: code.to_string(),
            message: msg,
        });
        (status, body).into_response()
    }
}

/// KPI counts per level
pub async fn get_summary_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.get_summary().await?))
}

/// One page of provinces (`search`, `active`, `page`, `limit`)
pub async fn list_provinces_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<GeoFilterQuery>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_provinces(filter).await?))
}

/// One page of districts (`province_id`, `search`, `active`, `page`, `limit`)
pub async fn list_districts_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<GeoFilterQuery>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_districts(filter).await?))
}

/// One page of sub-districts (`province_id`, `district_id`, `search`, `active`, `page`, `limit`)
pub async fn list_sub_districts_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<GeoFilterQuery>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_sub_districts(filter).await?))
}

/// One page of postal codes (`province_id`, `district_id`, `sub_district_id`, `search`, `active`, `page`, `limit`)
pub async fn list_postal_codes_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<PostalFilterQuery>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_postal_codes(filter).await?))
}

/// Get Province by ID
pub async fn get_province_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let found = commands
        .get_province(id)
        .await?
        .ok_or_else(|| Dbpg005Error::NotFound(format!("Province {id} not found")))?;
    Ok(wire.respond(found))
}

/// Get District by ID
pub async fn get_district_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let found = commands
        .get_district(id)
        .await?
        .ok_or_else(|| Dbpg005Error::NotFound(format!("District {id} not found")))?;
    Ok(wire.respond(found))
}

/// Get Sub-district by ID
pub async fn get_sub_district_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let found = commands
        .get_sub_district(id)
        .await?
        .ok_or_else(|| Dbpg005Error::NotFound(format!("Sub-district {id} not found")))?;
    Ok(wire.respond(found))
}

/// Get Postal code by ID
pub async fn get_postal_code_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let found = commands
        .get_postal_code(id)
        .await?
        .ok_or_else(|| Dbpg005Error::NotFound(format!("Postal code {id} not found")))?;
    Ok(wire.respond(found))
}

/// Create province
pub async fn create_province_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<CreateProvinceCommand>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.create_province(cmd, None).await?))
}

/// Create district
pub async fn create_district_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<CreateDistrictCommand>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.create_district(cmd, None).await?))
}

/// Create sub-district
pub async fn create_sub_district_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<CreateSubDistrictCommand>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.create_sub_district(cmd, None).await?))
}

/// Create postal code
pub async fn create_postal_code_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<CreatePostalCodeCommand>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.create_postal_code(cmd, None).await?))
}

/// Update province
pub async fn update_province_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
    WireBody(cmd): WireBody<UpdateGeoNodeCommand>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.update_province(id, cmd, None).await?))
}

/// Update district
pub async fn update_district_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
    WireBody(cmd): WireBody<UpdateGeoNodeCommand>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.update_district(id, cmd, None).await?))
}

/// Update sub-district
pub async fn update_sub_district_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
    WireBody(cmd): WireBody<UpdateGeoNodeCommand>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.update_sub_district(id, cmd, None).await?))
}

/// Update postal code
pub async fn update_postal_code_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
    WireBody(cmd): WireBody<UpdatePostalCodeCommand>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.update_postal_code(id, cmd, None).await?))
}

/// Delete province (only when nothing references it)
pub async fn delete_province_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Approve).await {
        return Ok(denied);
    }
    commands.delete_province(id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Delete district (only when nothing references it)
pub async fn delete_district_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Approve).await {
        return Ok(denied);
    }
    commands.delete_district(id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Delete sub-district (only when nothing references it)
pub async fn delete_sub_district_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Approve).await {
        return Ok(denied);
    }
    commands.delete_sub_district(id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Delete postal code (only when nothing references it)
pub async fn delete_postal_code_handler(
    State(commands): State<Arc<dyn Dbpg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Approve).await {
        return Ok(denied);
    }
    commands.delete_postal_code(id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Sub-router for DBPG005 endpoints mounted into `AppConnection`.
pub fn dbpg005_routes() -> Router<AppConnection> {
    Router::new()
        .route("/summary", get(get_summary_handler))
        .route("/provinces", get(list_provinces_handler).post(create_province_handler))
        .route("/provinces/{id}", get(get_province_handler).put(update_province_handler).delete(delete_province_handler))
        .route("/districts", get(list_districts_handler).post(create_district_handler))
        .route("/districts/{id}", get(get_district_handler).put(update_district_handler).delete(delete_district_handler))
        .route("/sub-districts", get(list_sub_districts_handler).post(create_sub_district_handler))
        .route("/sub-districts/{id}", get(get_sub_district_handler).put(update_sub_district_handler).delete(delete_sub_district_handler))
        .route("/postal-codes", get(list_postal_codes_handler).post(create_postal_code_handler))
        .route("/postal-codes/{id}", get(get_postal_code_handler).put(update_postal_code_handler).delete(delete_postal_code_handler))
}

/// Same routes with explicit state (useful for tests).
pub fn dbpg005_routes_with_state(state: Dbpg005State) -> Router {
    Router::new()
        .route("/summary", get(get_summary_handler))
        .route("/provinces", get(list_provinces_handler).post(create_province_handler))
        .route("/provinces/{id}", get(get_province_handler).put(update_province_handler).delete(delete_province_handler))
        .route("/districts", get(list_districts_handler).post(create_district_handler))
        .route("/districts/{id}", get(get_district_handler).put(update_district_handler).delete(delete_district_handler))
        .route("/sub-districts", get(list_sub_districts_handler).post(create_sub_district_handler))
        .route("/sub-districts/{id}", get(get_sub_district_handler).put(update_sub_district_handler).delete(delete_sub_district_handler))
        .route("/postal-codes", get(list_postal_codes_handler).post(create_postal_code_handler))
        .route("/postal-codes/{id}", get(get_postal_code_handler).put(update_postal_code_handler).delete(delete_postal_code_handler))
        .with_state(state)
}
