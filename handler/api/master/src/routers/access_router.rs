//! Access router: what the signed-in account may do (`GET /api/v1/access/me`).
//!
//! Mounted behind a Kratos session; no program permission is needed since everyone may ask about
//! their own rights. The web app uses the answer to hide the buttons an account cannot use; the
//! APIs still enforce the permissions themselves.

use application_master::features::su::access_control::AccessControl;
use axum::{
    Extension, Json, Router,
    extract::{FromRef, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use infrastructure::middleware::AuthenticatedUser;
use infrastructure::wire::WireFormat;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::username_of;
use crate::connection::AppConnection;

/// Path of the access API below the API prefix.
pub const ACCESS_PATH: &str = "/access";

/// State of the standalone router used by tests
#[derive(Clone)]
pub struct AccessState {
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<AccessState> for Arc<dyn AccessControl> {
    fn from_ref(s: &AccessState) -> Self {
        s.access.clone()
    }
}

/// The signed-in account's rights
#[derive(Serialize)]
pub struct AccessMeResponse {
    pub username: Option<String>,
    /// Program code -> granted actions (`READ` / `WRITE` / `APPROVE`)
    pub permissions: BTreeMap<String, BTreeSet<&'static str>>,
}

pub async fn access_me_handler(
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Response {
    let username = username_of(&user);
    let permissions = match &username {
        Some(name) => match access.permissions(name).await {
            Ok(p) => p,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "success": false, "code": "DATABASE_ERROR", "message": e.to_string() })),
                )
                    .into_response();
            }
        },
        None => BTreeMap::new(),
    };
    wire.respond(AccessMeResponse { username, permissions })
}

/// Sub-router mounted into `AppConnection`.
pub fn access_routes() -> Router<AppConnection> {
    Router::new().route("/me", get(access_me_handler))
}

/// Same routes with explicit state (useful for tests).
pub fn access_routes_with_state(state: AccessState) -> Router {
    Router::new().route("/me", get(access_me_handler)).with_state(state)
}
