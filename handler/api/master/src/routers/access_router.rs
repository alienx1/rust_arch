//! Access router: what the signed-in account may do (`GET /api/v1/access/me`).
//!
//! Mounted behind a Kratos session; no program permission is needed since everyone may ask about
//! their own rights. The web app uses the answer to hide the buttons an account cannot use; the
//! APIs still enforce the permissions themselves.

use application_master::features::su::access_control::{AccessControl, OwnBranch, OwnUnit};
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
    /// Photo of the employee behind the account, for the topbar avatar (`None` = no photo)
    pub image_url: Option<String>,
    /// Organization and company of the account's employee: the sidebar brand (name + logo)
    pub organization: Option<OwnUnit>,
    pub company: Option<OwnUnit>,
    /// Headquarters account: sees the whole organization (or the company it picks)
    pub is_hq: bool,
    /// Sees the whole organization (headquarters, or a group assigned at the organization)
    pub unlimited: bool,
    /// Companies the account may pick in the topbar switcher (the whole organization for
    /// headquarters, the own company otherwise)
    pub companies: Vec<OwnUnit>,
    /// Branches the switcher offers (the whole organization for headquarters, the own branch otherwise)
    pub branches: Vec<OwnBranch>,
    /// The branch the request is limited to (`None` = all branches)
    pub selected_branch_id: Option<uuid::Uuid>,
    /// The company the request is limited to (`None` = all companies: the overview)
    pub selected_company_id: Option<uuid::Uuid>,
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
    // A failed lookup only costs the avatar, never the rights
    let own = match &username {
        Some(name) => access.own_employee(name).await.ok().flatten(),
        None => None,
    };
    let (image_url, organization, company, is_hq, unlimited, companies, branches) = match own {
        Some(e) => (e.image_url, e.organization, e.company, e.is_hq, e.unlimited, e.choices, e.branches),
        None => (None, None, None, false, false, Vec::new(), Vec::new()),
    };
    let scope = application_core::view_scope::current();
    wire.respond(AccessMeResponse {
        username,
        image_url,
        organization,
        company,
        is_hq,
        unlimited,
        companies,
        branches,
        selected_branch_id: scope.as_ref().and_then(|s| s.single_branch()),
        selected_company_id: scope.as_ref().and_then(|s| s.single_company()),
        permissions,
    })
}

/// Sub-router mounted into `AppConnection`.
pub fn access_routes() -> Router<AppConnection> {
    Router::new().route("/me", get(access_me_handler))
}

/// Same routes with explicit state (useful for tests).
pub fn access_routes_with_state(state: AccessState) -> Router {
    Router::new().route("/me", get(access_me_handler)).with_state(state)
}
