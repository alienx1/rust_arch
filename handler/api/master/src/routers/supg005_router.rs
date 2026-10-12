//! SUPG005 router: IAM & Session Monitor.
//!
//! Lists enterprise users, inspects active sessions, suspends/reactivates accounts and revokes
//! user/device sessions in Ory Kratos. Mounted at `/api/v1/su/supg005` behind a Kratos session
//! (path = `db_program.path` of program `SUPG005`). Reading needs READ, suspending / reactivating
//! needs WRITE and revoking sessions needs APPROVE on `SUPG005`.

use application_master::features::su::supg005::{
    Supg005Commands, Supg005Error, RevokeResultDto, UserPaginationQuery,
};
use application_master::features::su::access_control::{AccessControl, PermissionAction};
use axum::{
    Extension, Json, Router,
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use infrastructure::middleware::AuthenticatedUser;
use serde::Serialize;
use std::sync::Arc;

use super::access::require;
use crate::connection::AppConnection;

const PROGRAM: &str = "SUPG005";

/// State of the standalone router used by tests (the app router takes both from `AppConnection`)
#[derive(Clone)]
pub struct Supg005State {
    pub commands: Arc<dyn Supg005Commands>,
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<Supg005State> for Arc<dyn Supg005Commands> {
    fn from_ref(s: &Supg005State) -> Self {
        s.commands.clone()
    }
}

impl FromRef<Supg005State> for Arc<dyn AccessControl> {
    fn from_ref(s: &Supg005State) -> Self {
        s.access.clone()
    }
}

/// Standardized error response body for IAM endpoints.
#[derive(Serialize)]
pub struct Supg005ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `Supg005Error`.
pub struct Supg005ApiError(pub Supg005Error);

impl From<Supg005Error> for Supg005ApiError {
    fn from(err: Supg005Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for Supg005ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            Supg005Error::UserNotFound(msg) => (StatusCode::NOT_FOUND, "USER_NOT_FOUND", msg.clone()),
            Supg005Error::Kratos(msg) => (
                StatusCode::BAD_GATEWAY,
                "KRATOS_ERROR",
                format!("Identity store error: {msg}"),
            ),
            Supg005Error::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                format!("Database error: {msg}"),
            ),
            Supg005Error::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                msg.clone(),
            ),
        };

        let body = Json(Supg005ErrorResponse {
            success: false,
            code: code.to_string(),
            message: msg,
        });

        (status, body).into_response()
    }
}

use infrastructure::wire::WireFormat;

/// Retrieve enterprise users along with active device sessions (paginated with SQL LIMIT/OFFSET).
pub async fn list_users(
    State(commands): State<Arc<dyn Supg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(params): Query<UserPaginationQuery>,
) -> Result<Response, Supg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let result = commands.list_users_paginated(params).await?;
    Ok(wire.respond(result))
}

/// Revoke all active sessions for a specific user.
pub async fn revoke_user_sessions(
    State(commands): State<Arc<dyn Supg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(user_id): Path<String>,
) -> Result<Response, Supg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Approve).await {
        return Ok(denied);
    }
    commands.revoke_user_sessions(&user_id).await?;
    Ok(wire.respond(RevokeResultDto {
        success: true,
        message: format!("Successfully revoked all sessions for user {user_id}"),
        target_id: user_id,
    }))
}

/// Suspend a user account (blocks login and revokes every session).
pub async fn suspend_user(
    State(commands): State<Arc<dyn Supg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(user_id): Path<String>,
) -> Result<Response, Supg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    commands.suspend_user(&user_id).await?;
    Ok(wire.respond(RevokeResultDto {
        success: true,
        message: format!("Successfully suspended user {user_id}"),
        target_id: user_id,
    }))
}

/// Lift a user account suspension.
pub async fn reactivate_user(
    State(commands): State<Arc<dyn Supg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(user_id): Path<String>,
) -> Result<Response, Supg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    commands.reactivate_user(&user_id).await?;
    Ok(wire.respond(RevokeResultDto {
        success: true,
        message: format!("Successfully reactivated user {user_id}"),
        target_id: user_id,
    }))
}

/// Revoke a specific device session.
pub async fn revoke_device_session(
    State(commands): State<Arc<dyn Supg005Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(session_id): Path<String>,
) -> Result<Response, Supg005ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Approve).await {
        return Ok(denied);
    }
    commands.revoke_device_session(&session_id).await?;
    Ok(wire.respond(RevokeResultDto {
        success: true,
        message: format!("Successfully revoked session {session_id}"),
        target_id: session_id,
    }))
}

/// Path of program `SUPG005` below the API prefix.
pub const SUPG005_PATH: &str = "/su/supg005";

/// Construct SUPG005 routes router.
pub fn supg005_routes() -> Router<AppConnection> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/{id}/sessions", delete(revoke_user_sessions))
        .route("/users/{id}/suspend", post(suspend_user))
        .route("/users/{id}/reactivate", post(reactivate_user))
        .route("/sessions/{id}", delete(revoke_device_session))
}

/// Builds the SUPG005 sub-router with explicit state (commands + access control).
/// Useful for modular composition and isolated unit testing.
pub fn supg005_routes_with_state(state: Supg005State) -> Router {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/{id}/sessions", delete(revoke_user_sessions))
        .route("/users/{id}/suspend", post(suspend_user))
        .route("/users/{id}/reactivate", post(reactivate_user))
        .route("/sessions/{id}", delete(revoke_device_session))
        .with_state(state)
}
