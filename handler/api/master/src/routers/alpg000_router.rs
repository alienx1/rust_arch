//! ALPG000 router: Audit Log search (events stored in Quickwit).
//!
//! Mounted at `/api/v1/al/alpg000` behind a Kratos session; every endpoint needs READ on program
//! `ALPG000`. A store that does not answer is reported as 503 `AUDIT_UNAVAILABLE`.

use application_master::features::al::alpg000::{Alpg000Commands, Alpg000Error, AuditLogFilterQuery};
use application_master::features::su::access_control::{AccessControl, PermissionAction};
use axum::{
    Extension, Json, Router,
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use infrastructure::middleware::AuthenticatedUser;
use infrastructure::wire::WireFormat;
use serde::Serialize;
use std::sync::Arc;

use super::access::require;
use crate::connection::AppConnection;

/// Path of program `ALPG000` below the API prefix.
pub const ALPG000_PATH: &str = "/al/alpg000";
const PROGRAM: &str = "ALPG000";

/// State of the standalone router used by tests (the app router takes both from `AppConnection`)
#[derive(Clone)]
pub struct Alpg000State {
    pub commands: Arc<dyn Alpg000Commands>,
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<Alpg000State> for Arc<dyn Alpg000Commands> {
    fn from_ref(s: &Alpg000State) -> Self {
        s.commands.clone()
    }
}

impl FromRef<Alpg000State> for Arc<dyn AccessControl> {
    fn from_ref(s: &Alpg000State) -> Self {
        s.access.clone()
    }
}

/// Standardized error response body for ALPG000 endpoints.
#[derive(Serialize)]
pub struct Alpg000ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `Alpg000Error`.
pub struct Alpg000ApiError(pub Alpg000Error);

impl From<Alpg000Error> for Alpg000ApiError {
    fn from(err: Alpg000Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for Alpg000ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            Alpg000Error::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
            Alpg000Error::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            Alpg000Error::Unavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "AUDIT_UNAVAILABLE",
                format!("Audit log service unavailable: {msg}"),
            ),
            Alpg000Error::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone())
            }
        };
        let body = Json(Alpg000ErrorResponse { success: false, code: code.to_string(), message: msg });
        (status, body).into_response()
    }
}

/// One page of events (`from`, `to`, `kind`, `user`, `program_code`, `method`, `status`,
/// `status_class`, `path`, `text`, `page`, `limit`)
pub async fn search_logs_handler(
    State(commands): State<Arc<dyn Alpg000Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<AuditLogFilterQuery>,
) -> Result<Response, Alpg000ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.search_logs(filter).await?))
}

/// Queues a CSV export of every event the filter matches (same query parameters as the list,
/// minus `page` / `limit`). The request goes to Redpanda, the report worker builds the file and
/// pushes a notification when it is ready: answers 202 with the job id at once.
pub async fn request_export_handler(
    State(commands): State<Arc<dyn Alpg000Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<AuditLogFilterQuery>,
) -> Result<Response, Alpg000ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let requested_by = user
        .username()
        .ok_or_else(|| Alpg000Error::Validation("The session has no username".to_string()))?;
    let queued = commands.request_export(&requested_by, filter).await?;
    let mut response = wire.respond(queued);
    *response.status_mut() = StatusCode::ACCEPTED;
    Ok(response)
}

/// One event with headers and bodies
pub async fn get_log_handler(
    State(commands): State<Arc<dyn Alpg000Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<String>,
) -> Result<Response, Alpg000ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let found = commands
        .get_log(&id)
        .await?
        .ok_or_else(|| Alpg000Error::NotFound(format!("Audit event {id} not found")))?;
    Ok(wire.respond(found))
}

/// Whether the audit store answers
pub async fn audit_status_handler(
    State(commands): State<Arc<dyn Alpg000Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Alpg000ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.status().await))
}

/// Catalogue programs the list can be filtered by
pub async fn program_options_handler(
    State(commands): State<Arc<dyn Alpg000Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Alpg000ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.program_options().await?))
}

/// Sub-router for ALPG000 endpoints mounted into `AppConnection`.
pub fn alpg000_routes() -> Router<AppConnection> {
    Router::new()
        .route("/logs", get(search_logs_handler))
        .route("/exports", post(request_export_handler))
        .route("/logs/{id}", get(get_log_handler))
        .route("/status", get(audit_status_handler))
        .route("/programs", get(program_options_handler))
}

/// Same routes with explicit state (useful for tests).
pub fn alpg000_routes_with_state(state: Alpg000State) -> Router {
    Router::new()
        .route("/logs", get(search_logs_handler))
        .route("/exports", post(request_export_handler))
        .route("/logs/{id}", get(get_log_handler))
        .route("/status", get(audit_status_handler))
        .route("/programs", get(program_options_handler))
        .with_state(state)
}
