//! SUPG001 router: Systems & Modules (`db_system` / `db_module`).
//!
//! Mounted at `/api/v1/su/supg001` behind a Kratos session. Reading needs READ, writing needs
//! WRITE and deleting needs APPROVE on program `SUPG001` (checked against the signed-in account's permission group).

use application_master::features::su::access_control::{AccessControl, PermissionAction};
use application_master::features::su::supg001::{
    CreateModuleCommand, CreateSystemCommand, ModuleFilterQuery, Supg001Commands, Supg001Error,
    SystemFilterQuery, UpdateUnitCommand,
};
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

/// Path of program `SUPG001` below the API prefix.
pub const SUPG001_PATH: &str = "/su/supg001";
const PROGRAM: &str = "SUPG001";

/// State of the standalone router used by tests (the app router takes both from `AppConnection`)
#[derive(Clone)]
pub struct Supg001State {
    pub commands: Arc<dyn Supg001Commands>,
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<Supg001State> for Arc<dyn Supg001Commands> {
    fn from_ref(s: &Supg001State) -> Self {
        s.commands.clone()
    }
}

impl FromRef<Supg001State> for Arc<dyn AccessControl> {
    fn from_ref(s: &Supg001State) -> Self {
        s.access.clone()
    }
}

/// Standardized error response body for SUPG001 endpoints.
#[derive(Serialize)]
pub struct Supg001ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `Supg001Error`.
pub struct Supg001ApiError(pub Supg001Error);

impl From<Supg001Error> for Supg001ApiError {
    fn from(err: Supg001Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for Supg001ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            Supg001Error::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            Supg001Error::DuplicateCode(msg) => {
                (StatusCode::CONFLICT, "DUPLICATE_CODE", msg.clone())
            }
            Supg001Error::VersionConflict => (
                StatusCode::PRECONDITION_FAILED,
                "VERSION_CONFLICT",
                "Data has been modified by another transaction. Please reload and try again."
                    .to_string(),
            ),
            Supg001Error::InUse(msg) => (StatusCode::CONFLICT, "IN_USE", msg.clone()),
            Supg001Error::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone())
            }
            Supg001Error::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                format!("Database error: {msg}"),
            ),
            Supg001Error::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone())
            }
        };

        let body = Json(Supg001ErrorResponse {
            success: false,
            code: code.to_string(),
            message: msg,
        });
        (status, body).into_response()
    }
}

/// Systems (`search`, `active`)
pub async fn list_systems_handler(
    State(commands): State<Arc<dyn Supg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<SystemFilterQuery>,
) -> Result<Response, Supg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_systems(filter).await?))
}

/// Modules (`system_id`, `search`, `active`)
pub async fn list_modules_handler(
    State(commands): State<Arc<dyn Supg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<ModuleFilterQuery>,
) -> Result<Response, Supg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_modules(filter).await?))
}

/// Get System by ID
pub async fn get_system_handler(
    State(commands): State<Arc<dyn Supg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Supg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let found = commands
        .get_system(id)
        .await?
        .ok_or_else(|| Supg001Error::NotFound(format!("System {id} not found")))?;
    Ok(wire.respond(found))
}

/// Get Module by ID
pub async fn get_module_handler(
    State(commands): State<Arc<dyn Supg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Supg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let found = commands
        .get_module(id)
        .await?
        .ok_or_else(|| Supg001Error::NotFound(format!("Module {id} not found")))?;
    Ok(wire.respond(found))
}

/// Create system
pub async fn create_system_handler(
    State(commands): State<Arc<dyn Supg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<CreateSystemCommand>,
) -> Result<Response, Supg001ApiError> {
    let actor = match require(&access, &user, PROGRAM, PermissionAction::Write).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    Ok(wire.respond(commands.create_system(cmd, Some(actor.user_id)).await?))
}

/// Create module
pub async fn create_module_handler(
    State(commands): State<Arc<dyn Supg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<CreateModuleCommand>,
) -> Result<Response, Supg001ApiError> {
    let actor = match require(&access, &user, PROGRAM, PermissionAction::Write).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    Ok(wire.respond(commands.create_module(cmd, Some(actor.user_id)).await?))
}

/// Update system
pub async fn update_system_handler(
    State(commands): State<Arc<dyn Supg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
    WireBody(cmd): WireBody<UpdateUnitCommand>,
) -> Result<Response, Supg001ApiError> {
    let actor = match require(&access, &user, PROGRAM, PermissionAction::Write).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    Ok(wire.respond(commands.update_system(id, cmd, Some(actor.user_id)).await?))
}

/// Update module
pub async fn update_module_handler(
    State(commands): State<Arc<dyn Supg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
    WireBody(cmd): WireBody<UpdateUnitCommand>,
) -> Result<Response, Supg001ApiError> {
    let actor = match require(&access, &user, PROGRAM, PermissionAction::Write).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    Ok(wire.respond(commands.update_module(id, cmd, Some(actor.user_id)).await?))
}

/// Delete system (hard delete; 409 `IN_USE` while still referenced)
pub async fn delete_system_handler(
    State(commands): State<Arc<dyn Supg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Response, Supg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Approve).await {
        return Ok(denied);
    }
    commands.delete_system(id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Delete module (hard delete; 409 `IN_USE` while still referenced)
pub async fn delete_module_handler(
    State(commands): State<Arc<dyn Supg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Response, Supg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Approve).await {
        return Ok(denied);
    }
    commands.delete_module(id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Sub-router for SUPG001 endpoints mounted into `AppConnection`.
pub fn supg001_routes() -> Router<AppConnection> {
    Router::new()
        .route("/systems", get(list_systems_handler).post(create_system_handler))
        .route("/systems/{id}", get(get_system_handler).put(update_system_handler).delete(delete_system_handler))
        .route("/modules", get(list_modules_handler).post(create_module_handler))
        .route("/modules/{id}", get(get_module_handler).put(update_module_handler).delete(delete_module_handler))
}

/// Same routes with explicit state (useful for tests).
pub fn supg001_routes_with_state(state: Supg001State) -> Router {
    Router::new()
        .route("/systems", get(list_systems_handler).post(create_system_handler))
        .route("/systems/{id}", get(get_system_handler).put(update_system_handler).delete(delete_system_handler))
        .route("/modules", get(list_modules_handler).post(create_module_handler))
        .route("/modules/{id}", get(get_module_handler).put(update_module_handler).delete(delete_module_handler))
        .with_state(state)
}
