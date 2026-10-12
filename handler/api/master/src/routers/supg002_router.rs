//! SUPG002 router: Menus & Programs — the sidebar menu tree and program management.
//!
//! Mounted at `/api/v1/su/supg002` behind a Kratos session (path = `db_program.path` of program
//! `SUPG002`).
//! - `GET /menu` builds the tree for the signed-in account: only active programs its permission
//!   group may READ, plus the folders above them (no extra permission needed).
//! - `/programs`, `/options`: management; reading needs READ, writing needs WRITE and deleting needs APPROVE on `SUPG002`.

use application_master::features::su::access_control::{AccessControl, PermissionAction};
use application_master::features::su::supg002::{
    CreateProgramCommand, ProgramFilterQuery, Supg002Commands, Supg002Error, UpdateProgramCommand,
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

/// Path of program `SUPG002` below the API prefix.
pub const SUPG002_PATH: &str = "/su/supg002";
const PROGRAM: &str = "SUPG002";

/// State of the standalone router used by tests (the app router takes both from `AppConnection`)
#[derive(Clone)]
pub struct Supg002State {
    pub commands: Arc<dyn Supg002Commands>,
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<Supg002State> for Arc<dyn Supg002Commands> {
    fn from_ref(s: &Supg002State) -> Self {
        s.commands.clone()
    }
}

impl FromRef<Supg002State> for Arc<dyn AccessControl> {
    fn from_ref(s: &Supg002State) -> Self {
        s.access.clone()
    }
}

/// Standardized error response body for SUPG002 endpoints.
#[derive(Serialize)]
pub struct Supg002ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `Supg002Error`.
pub struct Supg002ApiError(pub Supg002Error);

impl From<Supg002Error> for Supg002ApiError {
    fn from(err: Supg002Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for Supg002ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            Supg002Error::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            Supg002Error::DuplicateCode(msg) => (StatusCode::CONFLICT, "DUPLICATE_CODE", msg.clone()),
            Supg002Error::VersionConflict => (
                StatusCode::PRECONDITION_FAILED,
                "VERSION_CONFLICT",
                "Data has been modified by another transaction. Please reload and try again."
                    .to_string(),
            ),
            Supg002Error::InUse(msg) => (StatusCode::CONFLICT, "IN_USE", msg.clone()),
            Supg002Error::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
            Supg002Error::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                format!("Database error: {msg}"),
            ),
            Supg002Error::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone())
            }
        };
        let body = Json(Supg002ErrorResponse { success: false, code: code.to_string(), message: msg });
        (status, body).into_response()
    }
}

/// `su_user.username` of a Kratos identity: the `username` trait, else the local part of the email
/// (the same convention the login guard uses).
pub fn username_of(user: &AuthenticatedUser) -> Option<String> {
    user.username()
}

/// Menu tree for the signed-in account (`GET /menu`)
pub async fn get_menu_handler(
    State(commands): State<Arc<dyn Supg002Commands>>,
    wire: WireFormat,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Response, Supg002ApiError> {
    let tree = match username_of(&user) {
        Some(username) => commands.get_menu_tree(&username).await?,
        None => Vec::new(),
    };
    Ok(wire.respond(tree))
}

/// One page of programs (`system_id`, `module_id`, `search`, `active`, `page`, `limit`)
pub async fn list_programs_handler(
    State(commands): State<Arc<dyn Supg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<ProgramFilterQuery>,
) -> Result<Response, Supg002ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_programs(filter).await?))
}

/// Get program by ID
pub async fn get_program_handler(
    State(commands): State<Arc<dyn Supg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Supg002ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let found = commands
        .get_program(id)
        .await?
        .ok_or_else(|| Supg002Error::NotFound(format!("Program {id} not found")))?;
    Ok(wire.respond(found))
}

/// Systems, modules and programs for the form selectors
pub async fn program_options_handler(
    State(commands): State<Arc<dyn Supg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Supg002ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.program_options().await?))
}

/// Create program
pub async fn create_program_handler(
    State(commands): State<Arc<dyn Supg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<CreateProgramCommand>,
) -> Result<Response, Supg002ApiError> {
    let actor = match require(&access, &user, PROGRAM, PermissionAction::Write).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    Ok(wire.respond(commands.create_program(cmd, Some(actor.user_id)).await?))
}

/// Update program
pub async fn update_program_handler(
    State(commands): State<Arc<dyn Supg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
    WireBody(cmd): WireBody<UpdateProgramCommand>,
) -> Result<Response, Supg002ApiError> {
    let actor = match require(&access, &user, PROGRAM, PermissionAction::Write).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    Ok(wire.respond(commands.update_program(id, cmd, Some(actor.user_id)).await?))
}

/// Delete program (hard delete; 409 `IN_USE` while child programs still reference it)
pub async fn delete_program_handler(
    State(commands): State<Arc<dyn Supg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Response, Supg002ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Approve).await {
        return Ok(denied);
    }
    commands.delete_program(id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Sub-router for SUPG002 endpoints mounted into `AppConnection`.
pub fn supg002_routes() -> Router<AppConnection> {
    Router::new()
        .route("/menu", get(get_menu_handler))
        .route("/options", get(program_options_handler))
        .route("/programs", get(list_programs_handler).post(create_program_handler))
        .route("/programs/{id}", get(get_program_handler).put(update_program_handler).delete(delete_program_handler))
}

/// Same routes with explicit state (useful for tests).
pub fn supg002_routes_with_state(state: Supg002State) -> Router {
    Router::new()
        .route("/menu", get(get_menu_handler))
        .route("/options", get(program_options_handler))
        .route("/programs", get(list_programs_handler).post(create_program_handler))
        .route("/programs/{id}", get(get_program_handler).put(update_program_handler).delete(delete_program_handler))
        .with_state(state)
}
