//! Sign-in gateway in front of the Ory Kratos public API (`/api/v1/auth/kratos/{*path}`, no
//! session required: it is how a session is obtained).
//!
//! The web app talks to Kratos through this gateway instead of directly. Everything is passed
//! through unchanged except the password submit of the login flow (`POST self-service/login`),
//! where the gateway
//! - refuses the attempt while the account is locked (too many failed sign-ins), without asking
//!   Kratos, and tells the user for how long;
//! - counts an attempt Kratos answers with "credentials are invalid" (message `4000006`) as a
//!   failure, and locks the account when that was one too many.
//!
//! The limits are `security.login.*` in `su_configuration`. Kratos itself cannot count failed
//! sign-ins (its hooks run only after a success), hence the gateway. A reverse proxy in front of
//! the app must send `/self-service/*` here as well (see `docs/guides/PRODUCTION_CHECKLIST.md`).

use application_master::features::su::account_security::{AccountSecurityCommands, LoginState};
use axum::{
    Router,
    body::{Body, Bytes, to_bytes},
    extract::{FromRef, Path, Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::any,
};
use infrastructure::middleware::KratosConfig;
use std::sync::{Arc, OnceLock};

use crate::connection::AppConnection;

/// Path of the gateway below the API prefix
pub const AUTH_GATEWAY_PATH: &str = "/auth/kratos";

/// Largest request body the gateway reads (a login form is a few hundred bytes)
const MAX_BODY: usize = 256 * 1024;

/// Kratos message id: "The provided credentials are invalid ..."
const MSG_INVALID_CREDENTIALS: u64 = 4000006;
/// Our own message id for "locked"
const MSG_LOCKED: u64 = 4010001;

/// State of the gateway when it runs without `AppConnection` (tests)
#[derive(Clone)]
pub struct GatewayState {
    pub kratos_public_url: String,
    pub security: Arc<dyn AccountSecurityCommands>,
}

impl FromRef<GatewayState> for Arc<dyn AccountSecurityCommands> {
    fn from_ref(s: &GatewayState) -> Self {
        s.security.clone()
    }
}

impl FromRef<GatewayState> for GatewayTarget {
    fn from_ref(s: &GatewayState) -> Self {
        GatewayTarget(s.kratos_public_url.trim_end_matches('/').to_string())
    }
}

/// Where the gateway forwards to (the Kratos public URL)
#[derive(Clone)]
pub struct GatewayTarget(pub String);

impl FromRef<AppConnection> for GatewayTarget {
    fn from_ref(conn: &AppConnection) -> Self {
        let url = conn.kratos.as_ref().map(|k: &KratosConfig| k.public_url.clone()).unwrap_or_default();
        GatewayTarget(url.trim_end_matches('/').to_string())
    }
}

/// One client for every request: no redirects followed (the browser must see them), no cookie jar
fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("HTTP client for the Kratos gateway")
    })
}

/// Hop-by-hop headers and the ones the next hop recomputes
fn is_connection_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "connection" | "keep-alive" | "proxy-authenticate" | "proxy-authorization" | "te" | "trailer"
            | "transfer-encoding" | "upgrade" | "host" | "content-length" | "accept-encoding"
    )
}

fn bad_gateway(e: impl std::fmt::Display) -> Response {
    tracing::error!("Kratos gateway: {e}");
    (StatusCode::BAD_GATEWAY, "identity provider unreachable").into_response()
}

/// Sends the request to Kratos; `body` is what the caller already read
async fn forward(
    target: &str,
    method: &Method,
    path_and_query: &str,
    headers: &HeaderMap,
    body: Bytes,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut request = client().request(method.clone(), format!("{target}/{path_and_query}"));
    for (name, value) in headers {
        if !is_connection_header(name) {
            request = request.header(name, value);
        }
    }
    // The answer is read as text / JSON below: no compression
    request.header(header::ACCEPT_ENCODING, "identity").body(body).send().await
}

/// A Kratos answer as an axum response (all `Set-Cookie`s kept)
fn into_response(status: reqwest::StatusCode, headers: &reqwest::header::HeaderMap, body: Bytes) -> Response {
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    for (name, value) in headers {
        if is_connection_header(name) {
            continue;
        }
        if let Ok(value) = HeaderValue::from_bytes(value.as_bytes()) {
            response.headers_mut().append(name.clone(), value);
        }
    }
    response
}

/// `identifier` of a password submit (JSON or form body), and whether it is a password submit
fn password_submit(headers: &HeaderMap, body: &[u8]) -> Option<String> {
    let content_type = headers.get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or_default();
    let (identifier, method, has_password) = if content_type.contains("json") {
        let v: serde_json::Value = serde_json::from_slice(body).ok()?;
        (
            v["identifier"].as_str().map(str::to_string),
            v["method"].as_str().map(str::to_string),
            v.get("password").is_some(),
        )
    } else {
        let form: Vec<(String, String)> = serde_urlencoded::from_bytes(body).ok()?;
        let get = |k: &str| form.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone());
        (get("identifier"), get("method"), get("password").is_some())
    };
    let identifier = identifier.map(|i| i.trim().to_string()).filter(|i| !i.is_empty())?;
    (method.as_deref() == Some("password") || has_password).then_some(identifier)
}

/// Whether a Kratos flow JSON carries the "credentials are invalid" message
fn has_invalid_credentials(flow: &serde_json::Value) -> bool {
    fn walk(v: &serde_json::Value) -> bool {
        match v {
            serde_json::Value::Object(map) => {
                (map.get("id").and_then(|i| i.as_u64()) == Some(MSG_INVALID_CREDENTIALS)
                    && map.get("type").and_then(|t| t.as_str()) == Some("error"))
                    || map.values().any(walk)
            }
            serde_json::Value::Array(items) => items.iter().any(walk),
            _ => false,
        }
    }
    walk(flow)
}

fn locked_text(retry_after_secs: u64) -> String {
    let minutes = retry_after_secs.div_ceil(60).max(1);
    format!(
        "ลองเข้าสู่ระบบผิดหลายครั้ง บัญชีถูกล็อกชั่วคราว ลองใหม่ได้ในอีก {minutes} นาที / \
         Too many failed sign-in attempts. The account is locked: try again in {minutes} minute(s)."
    )
}

/// Adds the lock message to the flow's own messages (what the login page shows)
fn with_lock_message(mut flow: serde_json::Value, retry_after_secs: u64) -> serde_json::Value {
    let message = serde_json::json!({
        "id": MSG_LOCKED,
        "text": locked_text(retry_after_secs),
        "type": "error",
        "context": { "retry_after_seconds": retry_after_secs },
    });
    if let Some(ui) = flow.get_mut("ui").and_then(|u| u.as_object_mut()) {
        let messages = ui.entry("messages").or_insert_with(|| serde_json::json!([]));
        if let Some(list) = messages.as_array_mut() {
            list.retain(|m| m.get("id").and_then(|i| i.as_u64()) != Some(MSG_INVALID_CREDENTIALS));
            list.push(message);
        }
    }
    flow
}

fn json_response(status: StatusCode, value: &serde_json::Value, extra: Option<&reqwest::header::HeaderMap>) -> Response {
    let mut response = (status, axum::Json(value.clone())).into_response();
    if let Some(extra) = extra {
        for (name, value) in extra {
            if name == reqwest::header::SET_COOKIE
                && let Ok(v) = HeaderValue::from_bytes(value.as_bytes())
            {
                response.headers_mut().append(header::SET_COOKIE, v);
            }
        }
    }
    response
}

/// The answer to an attempt on a locked account: the login flow with the lock message, so the
/// page shows it like any other sign-in error
async fn locked_answer(target: &str, query: &str, headers: &HeaderMap, retry_after_secs: u64) -> Response {
    let flow_id = serde_urlencoded::from_str::<Vec<(String, String)>>(query)
        .ok()
        .and_then(|q| q.into_iter().find(|(k, _)| k == "flow").map(|(_, v)| v));
    if let Some(id) = flow_id
        && let Ok(res) = forward(target, &Method::GET, &format!("self-service/login/flows?id={id}"), headers, Bytes::new()).await
        && res.status().is_success()
    {
        let cookies = res.headers().clone();
        if let Ok(flow) = res.json::<serde_json::Value>().await {
            return json_response(StatusCode::BAD_REQUEST, &with_lock_message(flow, retry_after_secs), Some(&cookies));
        }
    }
    json_response(
        StatusCode::TOO_MANY_REQUESTS,
        &serde_json::json!({ "error": { "code": 429, "status": "Too Many Requests", "message": locked_text(retry_after_secs) } }),
        None,
    )
}

/// The gateway: every method, every path below `/auth/kratos/`
pub async fn gateway(
    State(target): State<GatewayTarget>,
    State(security): State<Arc<dyn AccountSecurityCommands>>,
    Path(path): Path<String>,
    request: Request,
) -> Response {
    if target.0.is_empty() {
        return (StatusCode::SERVICE_UNAVAILABLE, "authentication provider not configured").into_response();
    }
    let (parts, body) = request.into_parts();
    let query = parts.uri.query().unwrap_or_default().to_string();
    let path_and_query = if query.is_empty() { path.clone() } else { format!("{path}?{query}") };
    let body = match to_bytes(body, MAX_BODY).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::PAYLOAD_TOO_LARGE, "request too large").into_response(),
    };

    // Only the password submit of the login flow is watched
    let watched = (parts.method == Method::POST && path.trim_matches('/') == "self-service/login")
        .then(|| password_submit(&parts.headers, &body))
        .flatten();

    if let Some(identifier) = &watched {
        match security.login_state(identifier).await {
            Ok(LoginState::Locked { retry_after_secs }) => {
                return locked_answer(&target.0, &query, &parts.headers, retry_after_secs).await;
            }
            Ok(LoginState::Open) => {}
            // A lock store that cannot be read must not become a way around the lock
            Err(e) => {
                tracing::error!("sign-in lock unavailable: {e}");
                return (StatusCode::SERVICE_UNAVAILABLE, "sign-in temporarily unavailable").into_response();
            }
        }
    }

    let answer = match forward(&target.0, &parts.method, &path_and_query, &parts.headers, body).await {
        Ok(a) => a,
        Err(e) => return bad_gateway(e),
    };
    let (status, headers) = (answer.status(), answer.headers().clone());
    let bytes = match answer.bytes().await {
        Ok(b) => b,
        Err(e) => return bad_gateway(e),
    };

    let Some(identifier) = watched else {
        return into_response(status, &headers, bytes);
    };
    let is_json = headers
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("json"));
    let flow: Option<serde_json::Value> = is_json.then(|| serde_json::from_slice(&bytes).ok()).flatten();

    if status.is_success() || status.is_redirection() {
        // Signed in (the login hook already cleared the count; this covers a deployment without it)
        if let Err(e) = security.record_login_success(&identifier).await {
            tracing::warn!("failed sign-in counter not cleared: {e}");
        }
    } else if flow.as_ref().is_some_and(has_invalid_credentials) {
        match security.record_login_failure(&identifier).await {
            Ok(LoginState::Locked { retry_after_secs }) => {
                // This attempt was the last one allowed: the page says so instead of "invalid"
                if let Some(flow) = flow {
                    return json_response(status_of(status), &with_lock_message(flow, retry_after_secs), Some(&headers));
                }
            }
            Ok(LoginState::Open) => {}
            Err(e) => tracing::error!("failed sign-in not counted: {e}"),
        }
    }
    into_response(status, &headers, bytes)
}

fn status_of(status: reqwest::StatusCode) -> StatusCode {
    StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_REQUEST)
}

/// The gateway routes, mounted into `AppConnection` (public: they are how a session is obtained)
pub fn auth_gateway_routes() -> Router<AppConnection> {
    Router::new().route("/{*path}", any(gateway))
}

/// Same routes with explicit state (for isolated tests)
pub fn auth_gateway_routes_with_state(state: GatewayState) -> Router {
    Router::new().route("/{*path}", any(gateway)).with_state(state)
}
