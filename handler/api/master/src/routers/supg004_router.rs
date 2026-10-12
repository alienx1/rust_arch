//! SUPG004 router: User accounts (`su_user` + the Ory Kratos identity).
//!
//! Mounted at `/api/v1/su/supg004` behind a Kratos session. Reading needs READ, writing needs
//! WRITE and deleting needs APPROVE on program `SUPG004`; placing an account in the ADMIN group also needs APPROVE.

use application_master::features::su::access_control::{AccessControl, PermissionAction};
use application_master::features::su::supg004::{
    CreateUserCommand, EmployeeOptionsQuery, Supg004Commands, Supg004Error, UpdateUserCommand,
    UserFilterQuery,
};
use axum::{
    Extension, Json, Router,
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use infrastructure::middleware::AuthenticatedUser;
use infrastructure::wire::{WireBody, WireFormat};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use super::access::require;
use super::username_of;
use crate::connection::AppConnection;

/// Path of program `SUPG004` below the API prefix.
pub const SUPG004_PATH: &str = "/su/supg004";
const PROGRAM: &str = "SUPG004";

/// State of the standalone router used by tests (the app router takes both from `AppConnection`)
#[derive(Clone)]
pub struct Supg004State {
    pub commands: Arc<dyn Supg004Commands>,
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<Supg004State> for Arc<dyn Supg004Commands> {
    fn from_ref(s: &Supg004State) -> Self {
        s.commands.clone()
    }
}

impl FromRef<Supg004State> for Arc<dyn AccessControl> {
    fn from_ref(s: &Supg004State) -> Self {
        s.access.clone()
    }
}

/// Standardized error response body for SUPG004 endpoints.
#[derive(Serialize)]
pub struct Supg004ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `Supg004Error`.
pub struct Supg004ApiError(pub Supg004Error);

impl From<Supg004Error> for Supg004ApiError {
    fn from(err: Supg004Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for Supg004ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            Supg004Error::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            Supg004Error::Duplicate(msg) => (StatusCode::CONFLICT, "DUPLICATE", msg.clone()),
            Supg004Error::VersionConflict => (
                StatusCode::PRECONDITION_FAILED,
                "VERSION_CONFLICT",
                "Data has been modified by another transaction. Please reload and try again."
                    .to_string(),
            ),
            Supg004Error::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
            Supg004Error::NotAllowed(msg) => (StatusCode::FORBIDDEN, "NOT_ALLOWED", msg.clone()),
            Supg004Error::IdentityProvider(msg) => (
                StatusCode::BAD_GATEWAY,
                "IDENTITY_PROVIDER_ERROR",
                format!("Identity provider error: {msg}"),
            ),
            Supg004Error::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                format!("Database error: {msg}"),
            ),
            Supg004Error::InUse(msg) => (StatusCode::CONFLICT, "IN_USE", msg.clone()),
            Supg004Error::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone())
            }
        };
        let body = Json(Supg004ErrorResponse { success: false, code: code.to_string(), message: msg });
        (status, body).into_response()
    }
}

/// `true` when the caller holds APPROVE on SUPG004 (may place accounts in the ADMIN group)
async fn can_approve(access: &Arc<dyn AccessControl>, user: &AuthenticatedUser) -> bool {
    match username_of(user) {
        Some(username) => access
            .authorize(&username, PROGRAM, PermissionAction::Approve)
            .await
            .ok()
            .flatten()
            .is_some(),
        None => false,
    }
}

/// One page of accounts (`group_id`, `search`, `active`, `page`, `limit`)
pub async fn list_users_handler(
    State(commands): State<Arc<dyn Supg004Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<UserFilterQuery>,
) -> Result<Response, Supg004ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_users(filter).await?))
}

/// Get account by ID
pub async fn get_user_handler(
    State(commands): State<Arc<dyn Supg004Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Supg004ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let found = commands
        .get_user(id)
        .await?
        .ok_or_else(|| Supg004Error::NotFound(format!("User {id} not found")))?;
    Ok(wire.respond(found))
}

/// Employees that can still get an account (`search`)
pub async fn employee_options_handler(
    State(commands): State<Arc<dyn Supg004Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(query): Query<EmployeeOptionsQuery>,
) -> Result<Response, Supg004ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.employee_options(query).await?))
}

/// Permission groups an account can be placed in
pub async fn group_options_handler(
    State(commands): State<Arc<dyn Supg004Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Supg004ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.group_options().await?))
}

/// Organizations, companies, branches and departments for the assignment picker
pub async fn scope_options_handler(
    State(commands): State<Arc<dyn Supg004Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Supg004ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.scope_options().await?))
}

/// Create account (identity + local row); the response carries the one-time recovery link
pub async fn create_user_handler(
    State(commands): State<Arc<dyn Supg004Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<CreateUserCommand>,
) -> Result<Response, Supg004ApiError> {
    let actor = match require(&access, &user, PROGRAM, PermissionAction::Write).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    let approve = can_approve(&access, &user).await;
    Ok(wire.respond(commands.create_user(cmd, Some(actor.user_id), approve).await?))
}

/// Replace an account's assignments and/or switch it on/off
pub async fn update_user_handler(
    State(commands): State<Arc<dyn Supg004Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
    WireBody(cmd): WireBody<UpdateUserCommand>,
) -> Result<Response, Supg004ApiError> {
    let actor = match require(&access, &user, PROGRAM, PermissionAction::Write).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    let approve = can_approve(&access, &user).await;
    Ok(wire.respond(commands.update_user(id, cmd, Some(actor.user_id), approve).await?))
}

/// Delete an account and its sign-in identity (refused with 409 when still in use)
pub async fn delete_user_handler(
    State(commands): State<Arc<dyn Supg004Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Response, Supg004ApiError> {
    let actor = match require(&access, &user, PROGRAM, PermissionAction::Approve).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    commands.delete_user(id, Some(actor.user_id)).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Issue a one-time password-setting link for the account
pub async fn create_recovery_link_handler(
    State(commands): State<Arc<dyn Supg004Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Supg004ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.create_recovery_link(id).await?))
}

/// Sub-router for SUPG004 endpoints mounted into `AppConnection`.
pub fn supg004_routes() -> Router<AppConnection> {
    Router::new()
        .route("/options/employees", get(employee_options_handler))
        .route("/options/groups", get(group_options_handler))
        .route("/scope-options", get(scope_options_handler))
        .route("/users", get(list_users_handler).post(create_user_handler))
        .route("/users/{id}", get(get_user_handler).put(update_user_handler).delete(delete_user_handler))
        .route("/users/{id}/recovery-link", post(create_recovery_link_handler))
}

/// Same routes with explicit state (useful for tests).
pub fn supg004_routes_with_state(state: Supg004State) -> Router {
    Router::new()
        .route("/options/employees", get(employee_options_handler))
        .route("/options/groups", get(group_options_handler))
        .route("/scope-options", get(scope_options_handler))
        .route("/users", get(list_users_handler).post(create_user_handler))
        .route("/users/{id}", get(get_user_handler).put(update_user_handler).delete(delete_user_handler))
        .route("/users/{id}/recovery-link", post(create_recovery_link_handler))
        .with_state(state)
}
