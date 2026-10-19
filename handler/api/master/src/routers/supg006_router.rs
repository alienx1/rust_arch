//! SUPG006 router: security (brute-force lock) and mail template settings.
//!
//! Mounted at `/api/v1/su/supg006` behind a Kratos session. Reading needs READ and saving needs
//! WRITE on program `SUPG006`.

use application_master::features::su::access_control::{AccessControl, PermissionAction};
use application_master::features::su::supg006::{SaveConfigCommand, Supg006Commands, Supg006Error};
use axum::{
    Extension, Json, Router,
    extract::{FromRef, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use infrastructure::middleware::AuthenticatedUser;
use infrastructure::wire::{WireBody, WireFormat};
use serde::Serialize;
use std::sync::Arc;

use super::access::require;
use crate::connection::AppConnection;

/// Path of program `SUPG006` below the API prefix.
pub const SUPG006_PATH: &str = "/su/supg006";
const PROGRAM: &str = "SUPG006";

/// State of the standalone router used by tests (the app router takes both from `AppConnection`)
#[derive(Clone)]
pub struct Supg006State {
    pub commands: Arc<dyn Supg006Commands>,
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<Supg006State> for Arc<dyn Supg006Commands> {
    fn from_ref(s: &Supg006State) -> Self {
        s.commands.clone()
    }
}

impl FromRef<Supg006State> for Arc<dyn AccessControl> {
    fn from_ref(s: &Supg006State) -> Self {
        s.access.clone()
    }
}

/// Standardized error response body for SUPG006 endpoints.
#[derive(Serialize)]
pub struct Supg006ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `Supg006Error`.
pub struct Supg006ApiError(pub Supg006Error);

impl From<Supg006Error> for Supg006ApiError {
    fn from(err: Supg006Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for Supg006ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            Supg006Error::VersionConflict => (
                StatusCode::PRECONDITION_FAILED,
                "VERSION_CONFLICT",
                "Data has been modified by another transaction. Please reload and try again.".to_string(),
            ),
            Supg006Error::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
            Supg006Error::Database(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", format!("Database error: {msg}"))
            }
            Supg006Error::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone()),
        };
        (status, Json(Supg006ErrorResponse { success: false, code: code.to_string(), message: msg })).into_response()
    }
}

/// Every setting with its value and row version
pub async fn list_config_handler(
    State(commands): State<Arc<dyn Supg006Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Supg006ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_config().await?))
}

/// Save the changed settings (all or none); answers with the settings as stored
pub async fn save_config_handler(
    State(commands): State<Arc<dyn Supg006Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<SaveConfigCommand>,
) -> Result<Response, Supg006ApiError> {
    let actor = match require(&access, &user, PROGRAM, PermissionAction::Write).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    Ok(wire.respond(commands.save_config(cmd, Some(actor.user_id)).await?))
}

/// Sub-router for SUPG006 endpoints mounted into `AppConnection`.
pub fn supg006_routes() -> Router<AppConnection> {
    Router::new().route("/config", get(list_config_handler).put(save_config_handler))
}

/// Same routes with explicit state (useful for tests).
pub fn supg006_routes_with_state(state: Supg006State) -> Router {
    Router::new().route("/config", get(list_config_handler).put(save_config_handler)).with_state(state)
}
