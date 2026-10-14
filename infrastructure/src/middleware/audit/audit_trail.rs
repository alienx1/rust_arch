//! Audit trail: one `kind=api` event per API request (system log: who called what, with the
//! request and response, status and duration), handed to the `AuditLogPort`.
//!
//! - Who: the session middleware (`require_kratos_session`) puts the `AuthenticatedUser` into the
//!   extensions of the RESPONSE it returns (an outer layer cannot see request extensions), this
//!   middleware reads it from there. Requests rejected before the handler are recorded too, with
//!   no user. Routes that are not behind the session middleware (the DBPG* master-data APIs, the
//!   dashboard) have no such extension: for those the session cookie / token of the request is
//!   verified here instead (off the request path, through the same cached Kratos check), so the
//!   event still names the user. An invalid or missing session just stays anonymous.
//! - What is recorded / skipped: see `audit_target::should_audit`; bodies and headers: see
//!   `audit_body` (masking, CBOR -> JSON, size limits).
//! - Bodies are buffered only when their size is known up front and <= 256 KiB (content-length /
//!   exact size hint); anything else (chunked, streamed, larger) passes through untouched and is
//!   recorded as "<N bytes, not stored>".
//! - Data changes: for write methods the caller is resolved BEFORE the handler runs and handed to
//!   the database layer (`crate::with_actor`), so the `al_capture_change` trigger can
//!   record who changed which row; the `kind=api` event of the request shares its id as the
//!   change's `request_id`.
//! - Auditing never fails or slows a request: the event is built and queued in a spawned task and
//!   `record` itself never blocks.

use super::audit_body::{BODY_BUFFER_CAP, body_to_text, headers_to_json, mask_form, not_stored};
use super::audit_target::{derive_audit_target, should_audit};
use application_core::traits::{AuditEvent, AuditLogPort, audit_kind};
use axum::{
    body::{Body, Bytes},
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, header},
    middleware::Next,
    response::Response,
};
use http_body::Body as _;
use crate::middleware::auth::kratos::{AuthenticatedUser, KratosConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

/// State of the audit trail layer: where events go and, for routes without the session
/// middleware, how to find out who the caller was
#[derive(Clone)]
pub struct AuditTrailState {
    pub log: Arc<dyn AuditLogPort>,
    pub kratos: Option<KratosConfig>,
}

impl AuditTrailState {
    pub fn new(log: Arc<dyn AuditLogPort>, kratos: Option<KratosConfig>) -> Self {
        Self { log, kratos }
    }
}

/// Session credentials of a request: the `ory_kratos_session` cookie and / or `X-Session-Token`
fn session_credentials(headers: &HeaderMap) -> (Option<String>, Option<String>) {
    let token = headers
        .get("x-session-token")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let cookie = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').map(str::trim).find(|c| c.starts_with("ory_kratos_session="))
        })
        .map(str::to_string);
    (cookie, token)
}

/// A body as captured for the log
enum Captured {
    Bytes(Bytes),
    NotStored(String),
}

/// Buffers `body` when its size is known and within the cap; the returned body is equivalent to
/// the input either way
async fn capture(body: Body) -> (Body, Captured) {
    match body.size_hint().exact() {
        Some(n) if n as usize <= BODY_BUFFER_CAP => match axum::body::to_bytes(body, BODY_BUFFER_CAP).await {
            Ok(bytes) => (Body::from(bytes.clone()), Captured::Bytes(bytes)),
            Err(_) => (Body::empty(), Captured::NotStored("<unreadable body>".to_string())),
        },
        Some(n) => (body, Captured::NotStored(not_stored(n as usize))),
        None => (body, Captured::NotStored("<streamed body, not stored>".to_string())),
    }
}

fn captured_text(captured: &Captured, headers: &HeaderMap) -> Option<String> {
    match captured {
        Captured::Bytes(bytes) => body_to_text(
            headers.get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()),
            bytes,
        ),
        Captured::NotStored(text) => Some(text.clone()),
    }
}

/// First `X-Forwarded-For` hop, else the socket peer (needs `into_make_service_with_connect_info`)
fn client_ip(headers: &HeaderMap, peer: Option<SocketAddr>) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| peer.map(|p| p.ip().to_string()))
}

/// Primary language tag of `Accept-Language` (`th-TH,th;q=0.9` -> `th`)
fn primary_language(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .and_then(|v| v.split(';').next())
        .and_then(|v| v.trim().split('-').next())
        .map(str::to_ascii_lowercase)
        .filter(|s| !s.is_empty() && s != "*")
}

fn header_text(headers: &HeaderMap, name: header::HeaderName) -> Option<String> {
    headers.get(name).and_then(|v| v.to_str().ok()).map(str::to_string)
}

/// The caller of a write request, for the data-change audit; `None` when the session cannot be
/// verified (the handler will refuse the request anyway)
async fn write_actor(
    state: &AuditTrailState,
    headers: &HeaderMap,
    request_id: &str,
) -> Option<crate::AuditActor> {
    let kratos = state.kratos.as_ref()?;
    let (cookie, token) = session_credentials(headers);
    if cookie.is_none() && token.is_none() {
        return None;
    }
    let user = kratos.verify_session(cookie.as_deref(), token.as_deref()).await.ok()?;
    Some(crate::AuditActor {
        username: user.username().unwrap_or_default(),
        user_id: user.identity_id,
        request_id: request_id.to_string(),
    })
}

pub async fn audit_trail(
    State(state): State<AuditTrailState>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().as_str().to_string();
    let path = request.uri().path().to_string();
    if !should_audit(&method, &path) {
        return next.run(request).await;
    }

    let started = Instant::now();
    let query = request.uri().query().map(mask_form);
    let request_headers = request.headers().clone();
    let peer = request.extensions().get::<ConnectInfo<SocketAddr>>().map(|c| c.0);

    let event_id = uuid::Uuid::now_v7().to_string();
    let is_write = !matches!(method.as_str(), "GET" | "HEAD");
    let actor = if is_write { write_actor(&state, &request_headers, &event_id).await } else { None };

    let (parts, body) = request.into_parts();
    let (body, request_capture) = capture(body).await;
    let request = Request::from_parts(parts, body);
    let response = match actor {
        Some(actor) => crate::with_actor(actor, next.run(request)).await,
        None => next.run(request).await,
    };
    let duration_ms = started.elapsed().as_millis() as u64;

    let user = response.extensions().get::<AuthenticatedUser>().cloned();
    let (session_cookie, session_token) = session_credentials(&request_headers);
    let (mut response_parts, response_body) = response.into_parts();
    let (response_body, response_capture) = capture(response_body).await;
    let status = response_parts.status.as_u16();
    let response_headers = response_parts.headers.clone();
    // Keep the extensions (user attribution for any further outer layer)
    if let Some(u) = &user {
        response_parts.extensions.insert(u.clone());
    }

    // Decoding / masking happens off the request path
    let AuditTrailState { log, kratos } = state;
    tokio::spawn(async move {
        // No session middleware on this route: identify the caller from the session credentials
        let user = match (user, kratos) {
            (None, Some(kratos)) if session_cookie.is_some() || session_token.is_some() => kratos
                .verify_session(session_cookie.as_deref(), session_token.as_deref())
                .await
                .ok(),
            (user, _) => user,
        };
        let (program_code, action) = derive_audit_target(&method, &path);
        let event = AuditEvent {
            event_id,
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            kind: audit_kind::API.to_string(),
            user_id: user.as_ref().map(|u| u.identity_id.clone()),
            username: user.as_ref().and_then(AuthenticatedUser::username),
            email: user.as_ref().and_then(|u| u.email.clone()),
            session_id: user.as_ref().map(|u| u.session_id.clone()).filter(|s| !s.is_empty()),
            ip: client_ip(&request_headers, peer),
            user_agent: header_text(&request_headers, header::USER_AGENT),
            language: primary_language(&request_headers),
            method: Some(method),
            path: Some(path),
            query,
            program_code,
            action: Some(action),
            status: Some(status),
            duration_ms: Some(duration_ms),
            request_headers: Some(headers_to_json(&request_headers)),
            response_headers: Some(headers_to_json(&response_headers)),
            request_body: captured_text(&request_capture, &request_headers),
            response_body: captured_text(&response_capture, &response_headers),
            ..Default::default()
        };
        log.record(vec![event]);
    });

    Response::from_parts(response_parts, response_body)
}
