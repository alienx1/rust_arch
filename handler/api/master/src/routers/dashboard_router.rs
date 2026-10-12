//! Dashboard router for real-time aggregated master data metrics and platform overview.
//!
//! # Scope & Responsibilities
//! - Serves aggregated operational metrics and domain inventory counts.
//! - Consumes the `DashboardCommands` trait interface via Axum state.
//! - Returns `DashboardSummary` with KPIs, registered modules, and system statuses.

use application_master::features::dashboard::{
    DashboardCommands, DashboardError,
};
use application_master::features::su::access_control::{AccessControl, PermissionAction};
use axum::{
    Extension, Json, Router,
    extract::{FromRef, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use infrastructure::middleware::AuthenticatedUser;
use serde::Serialize;
use std::sync::Arc;

use super::access::require;
use crate::connection::AppConnection;

/// Program whose READ permission guards the dashboard.
const PROGRAM: &str = "SYPG000";

/// State of the standalone router used by tests (the app router takes both from `AppConnection`)
#[derive(Clone)]
pub struct DashboardState {
    pub commands: Arc<dyn DashboardCommands>,
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<DashboardState> for Arc<dyn DashboardCommands> {
    fn from_ref(s: &DashboardState) -> Self {
        s.commands.clone()
    }
}

impl FromRef<DashboardState> for Arc<dyn AccessControl> {
    fn from_ref(s: &DashboardState) -> Self {
        s.access.clone()
    }
}

/// Standardized error response body for dashboard endpoints.
#[derive(Serialize)]
pub struct DashboardErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `DashboardError`.
pub struct DashboardApiError(pub DashboardError);

impl From<DashboardError> for DashboardApiError {
    fn from(err: DashboardError) -> Self {
        Self(err)
    }
}

impl IntoResponse for DashboardApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            DashboardError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                msg.clone(),
            ),
            DashboardError::Database(msg) => {
                tracing::error!("Database error in dashboard: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "A database error occurred while fetching dashboard statistics".to_string(),
                )
            }
        };

        let body = Json(DashboardErrorResponse {
            success: false,
            code: code.to_string(),
            message: msg,
        });

        (status, body).into_response()
    }
}

use infrastructure::wire::WireFormat;

/// Handler for retrieving aggregated dashboard summary and KPIs.
///
/// Example: `GET /api/v1/master/dashboard/summary`
pub async fn get_dashboard_summary_handler(
    State(commands): State<Arc<dyn DashboardCommands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, DashboardApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let summary = commands.get_dashboard_summary().await?;
    Ok(wire.respond(summary))
}

/// Builds the dashboard sub-router.
///
/// Route provided:
/// - `GET /summary`
pub fn dashboard_routes() -> Router<AppConnection> {
    Router::new().route("/summary", get(get_dashboard_summary_handler))
}

/// Builds the dashboard sub-router with explicit state (commands + access control).
/// Useful for modular composition and isolated unit testing.
#[allow(dead_code)]
pub fn dashboard_routes_with_state(state: DashboardState) -> Router {
    Router::new()
        .route("/summary", get(get_dashboard_summary_handler))
        .with_state(state)
}
