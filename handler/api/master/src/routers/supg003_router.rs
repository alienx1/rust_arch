//! SUPG003 router: Permission groups and the permission matrix.
//!
//! Mounted at `/api/v1/su/supg003` behind a Kratos session. Reading needs READ, writing needs
//! WRITE and deleting needs APPROVE on program `SUPG003` (checked against the signed-in account's permission group).

use application_master::features::su::access_control::{AccessControl, PermissionAction};
use application_master::features::su::supg003::{
    CreatePermissionGroupCommand, GroupFilterQuery, SaveMatrixCommand, Supg003Commands,
    Supg003Error, UpdatePermissionGroupCommand,
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

/// Path of program `SUPG003` below the API prefix.
pub const SUPG003_PATH: &str = "/su/supg003";
const PROGRAM: &str = "SUPG003";

/// State of the standalone router used by tests (the app router takes both from `AppConnection`)
#[derive(Clone)]
pub struct Supg003State {
    pub commands: Arc<dyn Supg003Commands>,
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<Supg003State> for Arc<dyn Supg003Commands> {
    fn from_ref(s: &Supg003State) -> Self {
        s.commands.clone()
    }
}

impl FromRef<Supg003State> for Arc<dyn AccessControl> {
    fn from_ref(s: &Supg003State) -> Self {
        s.access.clone()
    }
}

/// Standardized error response body for SUPG003 endpoints.
#[derive(Serialize)]
pub struct Supg003ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `Supg003Error`.
pub struct Supg003ApiError(pub Supg003Error);

impl From<Supg003Error> for Supg003ApiError {
    fn from(err: Supg003Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for Supg003ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            Supg003Error::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            Supg003Error::DuplicateCode(msg) => {
                (StatusCode::CONFLICT, "DUPLICATE_CODE", msg.clone())
            }
            Supg003Error::VersionConflict => (
                StatusCode::PRECONDITION_FAILED,
                "VERSION_CONFLICT",
                "Data has been modified by another transaction. Please reload and try again."
                    .to_string(),
            ),
            Supg003Error::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone())
            }
            Supg003Error::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                format!("Database error: {msg}"),
            ),
            Supg003Error::InUse(msg) => (StatusCode::CONFLICT, "IN_USE", msg.clone()),
            Supg003Error::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone())
            }
        };

        let body = Json(Supg003ErrorResponse {
            success: false,
            code: code.to_string(),
            message: msg,
        });
        (status, body).into_response()
    }
}

/// Permission groups (`search`, `active`)
pub async fn list_groups_handler(
    State(commands): State<Arc<dyn Supg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<GroupFilterQuery>,
) -> Result<Response, Supg003ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_groups(filter).await?))
}

/// Get group by ID
pub async fn get_group_handler(
    State(commands): State<Arc<dyn Supg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Supg003ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let found = commands
        .get_group(id)
        .await?
        .ok_or_else(|| Supg003Error::NotFound(format!("Group {id} not found")))?;
    Ok(wire.respond(found))
}

/// The group's permission matrix
pub async fn get_matrix_handler(
    State(commands): State<Arc<dyn Supg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Supg003ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let found = commands
        .group_matrix(id)
        .await?
        .ok_or_else(|| Supg003Error::NotFound(format!("Group {id} not found")))?;
    Ok(wire.respond(found))
}

/// Create group
pub async fn create_group_handler(
    State(commands): State<Arc<dyn Supg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<CreatePermissionGroupCommand>,
) -> Result<Response, Supg003ApiError> {
    let actor = match require(&access, &user, PROGRAM, PermissionAction::Write).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    Ok(wire.respond(commands.create_group(cmd, Some(actor.user_id)).await?))
}

/// Switch a group on / off
pub async fn update_group_handler(
    State(commands): State<Arc<dyn Supg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
    WireBody(cmd): WireBody<UpdatePermissionGroupCommand>,
) -> Result<Response, Supg003ApiError> {
    let actor = match require(&access, &user, PROGRAM, PermissionAction::Write).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    Ok(wire.respond(commands.update_group(id, cmd, Some(actor.user_id)).await?))
}

/// Replace the group's matrix
pub async fn save_matrix_handler(
    State(commands): State<Arc<dyn Supg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
    WireBody(cmd): WireBody<SaveMatrixCommand>,
) -> Result<Response, Supg003ApiError> {
    let actor = match require(&access, &user, PROGRAM, PermissionAction::Write).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    Ok(wire.respond(commands.save_matrix(id, cmd, Some(actor.user_id)).await?))
}

/// Delete a group with its matrix (refused with 409 when still in use)
pub async fn delete_group_handler(
    State(commands): State<Arc<dyn Supg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Response, Supg003ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Approve).await {
        return Ok(denied);
    }
    commands.delete_group(id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Sub-router for SUPG003 endpoints mounted into `AppConnection`.
pub fn supg003_routes() -> Router<AppConnection> {
    Router::new()
        .route("/groups", get(list_groups_handler).post(create_group_handler))
        .route("/groups/{id}", get(get_group_handler).put(update_group_handler).delete(delete_group_handler))
        .route("/groups/{id}/matrix", get(get_matrix_handler).put(save_matrix_handler))
}

/// Same routes with explicit state (useful for tests).
pub fn supg003_routes_with_state(state: Supg003State) -> Router {
    Router::new()
        .route("/groups", get(list_groups_handler).post(create_group_handler))
        .route("/groups/{id}", get(get_group_handler).put(update_group_handler).delete(delete_group_handler))
        .route("/groups/{id}/matrix", get(get_matrix_handler).put(save_matrix_handler))
        .with_state(state)
}
