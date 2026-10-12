//! ALPG001 router: Process Monitor (background processes started through Redpanda).
//!
//! Mounted at `/api/v1/al/alpg001` behind a Kratos session. Listing and reading need READ on
//! program `ALPG001`; `reprocess` needs WRITE. A process that cannot be reprocessed now is 409, a
//! broker that does not answer is 503 `PROCESS_QUEUE_UNAVAILABLE`.

use application_master::features::al::alpg001::{
    Alpg001Commands, Alpg001Error, ProcessFilterQuery,
};
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

/// Path of program `ALPG001` below the API prefix.
pub const ALPG001_PATH: &str = "/al/alpg001";
const PROGRAM: &str = "ALPG001";

/// State of the standalone router used by tests (the app router takes both from `AppConnection`)
#[derive(Clone)]
pub struct Alpg001State {
    pub commands: Arc<dyn Alpg001Commands>,
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<Alpg001State> for Arc<dyn Alpg001Commands> {
    fn from_ref(s: &Alpg001State) -> Self {
        s.commands.clone()
    }
}

impl FromRef<Alpg001State> for Arc<dyn AccessControl> {
    fn from_ref(s: &Alpg001State) -> Self {
        s.access.clone()
    }
}

/// Standardized error response body for ALPG001 endpoints.
#[derive(Serialize)]
pub struct Alpg001ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `Alpg001Error`.
pub struct Alpg001ApiError(pub Alpg001Error);

impl From<Alpg001Error> for Alpg001ApiError {
    fn from(err: Alpg001Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for Alpg001ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            Alpg001Error::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
            Alpg001Error::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            Alpg001Error::Conflict(msg) => (StatusCode::CONFLICT, "PROCESS_NOT_REPROCESSABLE", msg.clone()),
            Alpg001Error::Unavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "PROCESS_QUEUE_UNAVAILABLE",
                format!("Process queue unavailable: {msg}"),
            ),
            Alpg001Error::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone())
            }
        };
        let body = Json(Alpg001ErrorResponse { success: false, code: code.to_string(), message: msg });
        (status, body).into_response()
    }
}

/// One page of processes (`status`, `kind`, `user`, `from`, `to`, `page`, `limit`) with the
/// per-status counters
pub async fn list_processes_handler(
    State(commands): State<Arc<dyn Alpg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<ProcessFilterQuery>,
) -> Result<Response, Alpg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_processes(filter).await?))
}

/// One process
pub async fn get_process_handler(
    State(commands): State<Arc<dyn Alpg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<String>,
) -> Result<Response, Alpg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let found = commands
        .get_process(&id)
        .await?
        .ok_or_else(|| Alpg001Error::NotFound(format!("Process {id} not found")))?;
    Ok(wire.respond(found))
}

/// Runs a process again from its stored request: 202 once it is queued
pub async fn reprocess_handler(
    State(commands): State<Arc<dyn Alpg001Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<String>,
) -> Result<Response, Alpg001ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Write).await {
        return Ok(denied);
    }
    let queued = commands.reprocess(&id).await?;
    let mut response = wire.respond(queued);
    *response.status_mut() = StatusCode::ACCEPTED;
    Ok(response)
}

/// Sub-router for ALPG001 endpoints mounted into `AppConnection`.
pub fn alpg001_routes() -> Router<AppConnection> {
    Router::new()
        .route("/processes", get(list_processes_handler))
        .route("/processes/{id}", get(get_process_handler))
        .route("/processes/{id}/reprocess", post(reprocess_handler))
}

/// Same routes with explicit state (useful for tests).
pub fn alpg001_routes_with_state(state: Alpg001State) -> Router {
    Router::new()
        .route("/processes", get(list_processes_handler))
        .route("/processes/{id}", get(get_process_handler))
        .route("/processes/{id}/reprocess", post(reprocess_handler))
        .with_state(state)
}
