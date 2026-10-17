//! Ory Kratos web hook router (not a program: called by Kratos, no session required).
//!
//! Mounted at `/api/v1/hooks`; the URLs are referenced by the Kratos config
//! (`kube/.../arch_identity`), so they stay stable independent of program paths.
//!
//! - `POST /login-guard` runs after Kratos has verified the password. It refuses an inactive
//!   account, and an account that must set a new password first (mailing the link again).
//! - `POST /password-changed` runs after a password was set (settings flow, which the emailed
//!   link leads to): the account may sign in again.

use application_master::features::su::account_security::AccountSecurityCommands;
use application_master::features::su::supg005::Supg005Commands;
use axum::{
    Json, Router,
    extract::{FromRef, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::connection::AppConnection;
use crate::routers::supg005_router::Supg005ApiError;

/// Body sent by the Kratos web hooks.
#[derive(Deserialize)]
pub struct HookRequest {
    /// The identity's e-mail (`traits.email`)
    pub identifier: String,
    /// The identity's user name (`traits.username`); the account is found by it, because the e-mail
    /// can differ from the user name once it was changed through the employee's contacts
    #[serde(default)]
    pub username: Option<String>,
}

impl HookRequest {
    /// The user name when the hook sends one (`None` when it is missing or Kratos left the
    /// template unrendered)
    fn sent_username(&self) -> Option<&str> {
        self.username.as_deref().map(str::trim).filter(|u| !u.is_empty() && !u.contains("no value"))
    }

    /// What names the account: the sent user name, else whatever the account security feature
    /// makes of the e-mail (it can differ from the user name, so a lookup may be needed)
    async fn account(&self, security: Option<&Arc<dyn AccountSecurityCommands>>) -> String {
        if let Some(u) = self.sent_username() {
            return u.to_string();
        }
        match security {
            Some(security) => security.account_name(&self.identifier).await,
            None => self.identifier.clone(),
        }
    }
}

/// State of the hook routes when they run without `AppConnection` (tests)
#[derive(Clone)]
pub struct HookState {
    pub supg005: Arc<dyn Supg005Commands>,
    pub security: Option<Arc<dyn AccountSecurityCommands>>,
}

impl FromRef<HookState> for Arc<dyn Supg005Commands> {
    fn from_ref(s: &HookState) -> Self {
        s.supg005.clone()
    }
}

impl FromRef<HookState> for Option<Arc<dyn AccountSecurityCommands>> {
    fn from_ref(s: &HookState) -> Self {
        s.security.clone()
    }
}

/// `Some(response)` when the call does not carry the shared secret (`KRATOS_WEBHOOK_SECRET`);
/// fails closed with 503 when the secret is not configured.
fn reject_unauthenticated(headers: &HeaderMap) -> Option<Response> {
    let Some(secret) = std::env::var("KRATOS_WEBHOOK_SECRET").ok().filter(|s| !s.is_empty()) else {
        return Some((StatusCode::SERVICE_UNAVAILABLE, "login guard secret not configured").into_response());
    };
    let provided = headers.get("x-webhook-secret").and_then(|v| v.to_str().ok());
    // Not `!=`: compare in constant time so the secret cannot be guessed byte by byte
    if !constant_time_eq(provided.unwrap_or_default().as_bytes(), secret.as_bytes()) {
        return Some(StatusCode::UNAUTHORIZED.into_response());
    }
    None
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    use subtle::ConstantTimeEq;
    a.len() == b.len() && a.ct_eq(b).into()
}

/// Kratos error format for parsed web hook responses.
fn refusal(id: u32, text: &str) -> Response {
    let body = serde_json::json!({
        "messages": [{
            "instance_ptr": "#/",
            "messages": [{ "id": id, "text": text, "type": "error" }]
        }]
    });
    (StatusCode::FORBIDDEN, Json(body)).into_response()
}

/// Kratos after-login web hook: refuse the login when `su_user.active` is false, or when the
/// account must set a new password first.
pub async fn login_guard(
    State(commands): State<Arc<dyn Supg005Commands>>,
    State(security): State<Option<Arc<dyn AccountSecurityCommands>>>,
    headers: HeaderMap,
    Json(req): Json<HookRequest>,
) -> Result<Response, Supg005ApiError> {
    if let Some(rejected) = reject_unauthenticated(&headers) {
        return Ok(rejected);
    }

    let account = req.account(security.as_ref()).await;
    if !commands.authorize_login(&account).await? {
        return Ok(refusal(4000001, "This account, its company or its organization is inactive."));
    }
    if let Some(security) = security {
        match security.password_change_required(&account).await {
            Ok(true) => {
                // The password was right, but a new one must be set first: mail the link again
                // (throttled inside) without making the sign-in wait for the mail server
                let identifier = account.clone();
                tokio::spawn(async move { security.notify_password_required(&identifier).await });
                return Ok(refusal(
                    4010002,
                    "ต้องตั้งรหัสผ่านใหม่ก่อนเข้าใช้งาน ระบบส่งลิงก์ไปที่อีเมลของคุณแล้ว / You must set a new password before signing in: a link was sent to your e-mail.",
                ));
            }
            Ok(false) => {
                // A right password ends the run of failed attempts
                if let Err(e) = security.record_login_success(&account).await {
                    tracing::warn!("failed sign-in counter not cleared: {e}");
                }
            }
            // Never let an account through that might have to change its password
            Err(e) => {
                tracing::error!("password-change check failed: {e}");
                return Ok((StatusCode::SERVICE_UNAVAILABLE, "login guard unavailable").into_response());
            }
        }
    }
    // The hook is configured with `response.parse: true`, so Kratos requires a JSON body.
    Ok((StatusCode::OK, Json(serde_json::json!({}))).into_response())
}

/// Kratos after-settings (password) web hook: a new password is set, the account may sign in.
pub async fn password_changed(
    State(security): State<Option<Arc<dyn AccountSecurityCommands>>>,
    headers: HeaderMap,
    Json(req): Json<HookRequest>,
) -> Response {
    if let Some(rejected) = reject_unauthenticated(&headers) {
        return rejected;
    }
    let account = req.account(security.as_ref()).await;
    if let Some(security) = security
        && let Err(e) = security.mark_password_changed(&account).await
    {
        tracing::error!("password-changed hook could not clear the flag: {e}");
        return (StatusCode::SERVICE_UNAVAILABLE, "could not record the password change").into_response();
    }
    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
}

/// Routes called by Ory Kratos (not browser-facing, no session required).
pub fn kratos_hook_routes() -> Router<AppConnection> {
    Router::new()
        .route("/login-guard", post(login_guard))
        .route("/password-changed", post(password_changed))
}

/// Hook routes with explicit state (for isolated tests).
pub fn kratos_hook_routes_with_state(state: HookState) -> Router {
    Router::new()
        .route("/login-guard", post(login_guard))
        .route("/password-changed", post(password_changed))
        .with_state(state)
}

/// Hook routes with just the IAM commands (the login guard without the password rules)
pub fn kratos_hook_routes_with_commands(commands: Arc<dyn Supg005Commands>) -> Router {
    kratos_hook_routes_with_state(HookState { supg005: commands, security: None })
}
