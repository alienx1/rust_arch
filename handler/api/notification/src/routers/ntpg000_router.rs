//! NTPG000 router: the signed-in user's notifications, "mark as read", and the server-sent-event
//! stream that pushes new ones to every open session of the user.
//!
//! Mounted at `/api/v1/notifications` behind a Kratos session. Everyone sees only their own
//! notifications (somebody else's is a 404).

use application_core::traits::PushEvent;
use application_notification::features::ntpg000::{Ntpg000Commands, Ntpg000Error};
use axum::{
    Extension, Json, Router,
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, post},
};
use futures_util::{StreamExt, stream};
use infrastructure::middleware::AuthenticatedUser;
use infrastructure::services::SseNotificationHub;
use infrastructure::wire::WireFormat;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;

use crate::connection::NotificationConnection;

impl FromRef<NotificationConnection> for Arc<dyn Ntpg000Commands> {
    fn from_ref(c: &NotificationConnection) -> Self {
        c.ntpg000_commands.clone()
    }
}

impl FromRef<NotificationConnection> for Arc<SseNotificationHub> {
    fn from_ref(c: &NotificationConnection) -> Self {
        c.sessions.clone()
    }
}

#[derive(Serialize)]
pub struct Ntpg000ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

pub struct Ntpg000ApiError(pub Ntpg000Error);

impl From<Ntpg000Error> for Ntpg000ApiError {
    fn from(e: Ntpg000Error) -> Self {
        Self(e)
    }
}

impl IntoResponse for Ntpg000ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self.0 {
            Ntpg000Error::NotFound(m) => (StatusCode::NOT_FOUND, "NOT_FOUND", m.clone()),
            Ntpg000Error::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", m.clone()),
        };
        (status, Json(Ntpg000ErrorResponse { success: false, code: code.to_string(), message })).into_response()
    }
}

fn username(user: &AuthenticatedUser) -> Result<String, Ntpg000ApiError> {
    user.username()
        .ok_or_else(|| Ntpg000ApiError(Ntpg000Error::NotFound("The session has no username".into())))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<u64>,
}

/// `GET /items?limit=`: my notifications, newest first, with the unread count
pub async fn list_handler(
    State(commands): State<Arc<dyn Ntpg000Commands>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(q): Query<ListQuery>,
) -> Result<Response, Ntpg000ApiError> {
    Ok(wire.respond(commands.list(&username(&user)?, q.limit).await?))
}

/// `POST /{id}/read`
pub async fn mark_read_handler(
    State(commands): State<Arc<dyn Ntpg000Commands>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> Result<StatusCode, Ntpg000ApiError> {
    commands.mark_read(&username(&user)?, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /read-all`
pub async fn mark_all_read_handler(
    State(commands): State<Arc<dyn Ntpg000Commands>>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<StatusCode, Ntpg000ApiError> {
    commands.mark_all_read(&username(&user)?).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /stream`: server-sent events. `notification` = a new one (the notification as JSON),
/// `changed` = read state changed elsewhere (reload the list).
pub async fn stream_handler(
    State(hub): State<Arc<SseNotificationHub>>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, Ntpg000ApiError> {
    let receiver = hub.subscribe(&username(&user)?);
    // First event right away so the browser knows the stream is open
    let hello = stream::once(async { Ok(Event::default().event("ready").data("{}")) });
    let pushes = stream::unfold(receiver, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(PushEvent::Notification(n)) => {
                    let event = Event::default().event("notification").json_data(&n).unwrap_or_default();
                    return Some((Ok(event), rx));
                }
                // A notice that is not stored: the page shows it, nothing to reload
                Ok(PushEvent::Live(n)) => {
                    let event = Event::default().event("live").json_data(&n).unwrap_or_default();
                    return Some((Ok(event), rx));
                }
                Ok(PushEvent::Changed) => return Some((Ok(Event::default().event("changed").data("{}")), rx)),
                // Fell behind: tell the tab to reload instead of guessing what it missed
                Err(RecvError::Lagged(_)) => return Some((Ok(Event::default().event("changed").data("{}")), rx)),
                Err(RecvError::Closed) => return None,
            }
        }
    });
    Ok(Sse::new(hello.chain(pushes)).keep_alive(KeepAlive::default()))
}

pub fn ntpg000_routes() -> Router<NotificationConnection> {
    Router::new()
        .route("/items", get(list_handler))
        .route("/read-all", post(mark_all_read_handler))
        .route("/{id}/read", post(mark_read_handler))
        .route("/stream", get(stream_handler))
}
