//! Ory Kratos web hook router (not a program: called by Kratos, no session required).
//!
//! Mounted at `/api/v1/hooks`; the URL is referenced by the Kratos config
//! (`kube/.../arch_identity`), so it stays stable independent of program paths.

use application_master::features::su::supg005::Supg005Commands;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::connection::AppConnection;
use crate::routers::supg005_router::Supg005ApiError;

/// Body sent by the Kratos after-login web hook.
#[derive(Deserialize)]
pub struct LoginGuardRequest {
    pub identifier: String,
}

/// Kratos after-login web hook: refuse the login when `su_user.active` is false.
///
/// Authenticated with the shared secret in `KRATOS_WEBHOOK_SECRET` (`X-Webhook-Secret` header);
/// fails closed with 503 when the secret is not configured.
pub async fn login_guard(
    State(commands): State<Arc<dyn Supg005Commands>>,
    headers: HeaderMap,
    Json(req): Json<LoginGuardRequest>,
) -> Result<Response, Supg005ApiError> {
    let Some(secret) = std::env::var("KRATOS_WEBHOOK_SECRET").ok().filter(|s| !s.is_empty()) else {
        return Ok((StatusCode::SERVICE_UNAVAILABLE, "login guard secret not configured").into_response());
    };
    let provided = headers.get("x-webhook-secret").and_then(|v| v.to_str().ok());
    if provided != Some(secret.as_str()) {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    if commands.authorize_login(&req.identifier).await? {
        // The hook is configured with `response.parse: true`, so Kratos requires a JSON body.
        return Ok((StatusCode::OK, Json(serde_json::json!({}))).into_response());
    }
    // Kratos error format for parsed web hook responses.
    let body = serde_json::json!({
        "messages": [{
            "instance_ptr": "#/",
            "messages": [{ "id": 4000001, "text": "This account, its company or its organization is inactive.", "type": "error" }]
        }]
    });
    Ok((StatusCode::FORBIDDEN, Json(body)).into_response())
}

/// Routes called by Ory Kratos (not browser-facing, no session required).
pub fn kratos_hook_routes() -> Router<AppConnection> {
    Router::new().route("/login-guard", post(login_guard))
}

/// Hook routes with explicit `Arc<dyn Supg005Commands>` state (for isolated tests).
pub fn kratos_hook_routes_with_commands(commands: Arc<dyn Supg005Commands>) -> Router {
    Router::new()
        .route("/login-guard", post(login_guard))
        .with_state(commands)
}

