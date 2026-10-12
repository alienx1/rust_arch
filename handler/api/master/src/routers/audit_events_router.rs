//! Audit events intake: the web app reports the pages a user opens, the buttons pressed and the
//! processes run (`POST /api/v1/audit/events`).
//!
//! Behind a Kratos session only: every signed-in user reports their own activity, so there is no
//! program permission. The actor (user, session, ip, user agent, language) is taken from the
//! session and the request headers, never from the body. Answers 204; 400 only for a malformed
//! body or more than 50 events.

use application_master::features::al::alpg000::{
    Alpg000Commands, ClientActor, ClientEventCommand,
};
use axum::{
    Extension, Router,
    extract::{FromRef, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use infrastructure::middleware::AuthenticatedUser;
use infrastructure::wire::WireBody;
use serde::Deserialize;
use std::sync::Arc;

use super::alpg000_router::Alpg000ApiError;
use super::username_of;
use crate::connection::AppConnection;

/// Path of the intake below the API prefix.
pub const AUDIT_EVENTS_PATH: &str = "/audit";

/// State of the standalone router used by tests
#[derive(Clone)]
pub struct AuditEventsState {
    pub commands: Arc<dyn Alpg000Commands>,
}

impl FromRef<AuditEventsState> for Arc<dyn Alpg000Commands> {
    fn from_ref(s: &AuditEventsState) -> Self {
        s.commands.clone()
    }
}

/// Request body of `POST /events`
#[derive(Debug, Deserialize)]
pub struct AuditEventsRequest {
    #[serde(default)]
    pub events: Vec<ClientEventCommand>,
}

/// Who is calling: session identity + `X-Forwarded-For` (first hop), `User-Agent`, `Accept-Language`
pub fn client_actor_of(user: &AuthenticatedUser, headers: &HeaderMap) -> ClientActor {
    let header = |name: &str| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
    };
    ClientActor {
        user_id: Some(user.identity_id.clone()),
        username: username_of(user),
        email: user.email.clone(),
        session_id: Some(user.session_id.clone()),
        ip: header("x-forwarded-for")
            .and_then(|v| v.split(',').next().map(|h| h.trim().to_string()))
            .filter(|h| !h.is_empty()),
        user_agent: header("user-agent"),
        language: header("accept-language").and_then(|v| {
            v.split(',')
                .next()
                .and_then(|t| t.split(['-', ';']).next())
                .map(|t| t.trim().to_lowercase())
                .filter(|t| !t.is_empty())
        }),
    }
}

/// Stores the reported events
pub async fn record_events_handler(
    State(commands): State<Arc<dyn Alpg000Commands>>,
    Extension(user): Extension<AuthenticatedUser>,
    headers: HeaderMap,
    WireBody(request): WireBody<AuditEventsRequest>,
) -> Result<Response, Alpg000ApiError> {
    let actor = client_actor_of(&user, &headers);
    commands.record_client_events(actor, request.events).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Sub-router mounted into `AppConnection`.
pub fn audit_events_routes() -> Router<AppConnection> {
    Router::new().route("/events", post(record_events_handler))
}

/// Same routes with explicit state (useful for tests).
pub fn audit_events_routes_with_state(state: AuditEventsState) -> Router {
    Router::new().route("/events", post(record_events_handler)).with_state(state)
}
