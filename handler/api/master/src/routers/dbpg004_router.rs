//! DBPG004 router: Reference Data (`db_master_lookup`): paged list per category + create/update.
//!
//! Mounted at `/api/v1/db/dbpg004` (path = `db_program.path` of program `DBPG004`).

use application_master::features::db::dbpg004::{
    CreateLookupCommand, Dbpg004Commands, Dbpg004Error, LookupFilterQuery, UpdateLookupCommand,
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

/// Path of program `DBPG004` below the API prefix.
pub const DBPG004_PATH: &str = "/db/dbpg004";
const PROGRAM: &str = "DBPG004";

/// State of the standalone router used by tests (the app router takes both from `AppConnection`)
#[derive(Clone)]
pub struct Dbpg004State {
    pub commands: Arc<dyn Dbpg004Commands>,
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<Dbpg004State> for Arc<dyn Dbpg004Commands> {
    fn from_ref(s: &Dbpg004State) -> Self {
        s.commands.clone()
    }
}

impl FromRef<Dbpg004State> for Arc<dyn AccessControl> {
    fn from_ref(s: &Dbpg004State) -> Self {
        s.access.clone()
    }
}

/// Standardized error response body for DBPG004 endpoints.
#[derive(Serialize)]
pub struct Dbpg004ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `Dbpg004Error`.
pub struct Dbpg004ApiError(pub Dbpg004Error);

impl From<Dbpg004Error> for Dbpg004ApiError {
    fn from(err: Dbpg004Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for Dbpg004ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            Dbpg004Error::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            Dbpg004Error::DuplicateCode(msg) => {
                (StatusCode::CONFLICT, "DUPLICATE_CODE", msg.clone())
            }
            Dbpg004Error::VersionConflict => (
                StatusCode::PRECONDITION_FAILED,
                "VERSION_CONFLICT",
                "Data has been modified by another transaction. Please reload and try again."
                    .to_string(),
            ),
            Dbpg004Error::InUse(msg) => (StatusCode::CONFLICT, "IN_USE", msg.clone()),
            Dbpg004Error::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone())
            }
            Dbpg004Error::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                format!("Database error: {msg}"),
            ),
            Dbpg004Error::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone())
            }
        };

        let body = Json(Dbpg004ErrorResponse {
            success: false,
            code: code.to_string(),
            message: msg,
        });
        (status, body).into_response()
    }
}

/// Categories with row counts
pub async fn list_categories_handler(
    State(commands): State<Arc<dyn Dbpg004Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg004ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_categories().await?))
}

/// KPI summary
pub async fn get_summary_handler(
    State(commands): State<Arc<dyn Dbpg004Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg004ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.get_summary().await?))
}

/// One page of lookups (`category`, `search`, `active`, `page`, `limit`)
pub async fn list_lookups_handler(
    State(commands): State<Arc<dyn Dbpg004Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<LookupFilterQuery>,
) -> Result<Response, Dbpg004ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_lookups(filter).await?))
}

/// Get lookup by ID
pub async fn get_lookup_handler(
    State(commands): State<Arc<dyn Dbpg004Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg004ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let lookup = commands
        .get_lookup_by_id(id)
        .await?
        .ok_or_else(|| Dbpg004Error::NotFound(format!("Lookup {id} not found")))?;
    Ok(wire.respond(lookup))
}

/// Create lookup
pub async fn create_lookup_handler(
    State(commands): State<Arc<dyn Dbpg004Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<CreateLookupCommand>,
) -> Result<Response, Dbpg004ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.create_lookup(cmd, None).await?))
}

/// Update lookup
pub async fn update_lookup_handler(
    State(commands): State<Arc<dyn Dbpg004Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
    WireBody(cmd): WireBody<UpdateLookupCommand>,
) -> Result<Response, Dbpg004ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.update_lookup(id, cmd, None).await?))
}

/// Delete lookup (hard delete; 409 `IN_USE` while still referenced)
pub async fn delete_lookup_handler(
    State(commands): State<Arc<dyn Dbpg004Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg004ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Approve).await {
        return Ok(denied);
    }
    commands.delete_lookup(id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Sub-router for DBPG004 endpoints mounted into `AppConnection`.
pub fn dbpg004_routes() -> Router<AppConnection> {
    Router::new()
        .route("/categories", get(list_categories_handler))
        .route("/summary", get(get_summary_handler))
        .route("/lookups", get(list_lookups_handler).post(create_lookup_handler))
        .route("/lookups/{id}", get(get_lookup_handler).put(update_lookup_handler).delete(delete_lookup_handler))
}

/// Same routes with explicit state (useful for tests).
pub fn dbpg004_routes_with_state(state: Dbpg004State) -> Router {
    Router::new()
        .route("/categories", get(list_categories_handler))
        .route("/summary", get(get_summary_handler))
        .route("/lookups", get(list_lookups_handler).post(create_lookup_handler))
        .route("/lookups/{id}", get(get_lookup_handler).put(update_lookup_handler).delete(delete_lookup_handler))
        .with_state(state)
}
