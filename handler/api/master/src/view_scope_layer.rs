//! Works out what the request may look at and runs the rest of it inside that scope
//! (`application_core::view_scope`).
//!
//! A headquarters account (or one holding a permission group at the whole organization) sees the
//! organization, or the company (header [`COMPANY_HEADER`]) or branch (header [`BRANCH_HEADER`]) it
//! picked in the sidebar switcher. Any other account sees its own branch plus the branches its
//! permission groups reach, and a pick narrows that, never widens it. Mounted behind the session
//! middleware (it needs the signed-in user); an account that cannot be placed (unknown, no
//! employee) is not narrowed.

use std::sync::Arc;

use application_master::features::su::access_control::{AccessControl, PermissionAction, view_scope_for};
use axum::{
    Json,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use infrastructure::middleware::{AuthenticatedUser, derive_audit_target};
use uuid::Uuid;

/// Header the web app sends with the company chosen in the sidebar switcher
pub const COMPANY_HEADER: &str = "x-company-id";
/// ... and the branch chosen under it
pub const BRANCH_HEADER: &str = "x-branch-id";

pub async fn apply_view_scope(
    State(access): State<Arc<dyn AccessControl>>,
    req: Request,
    next: Next,
) -> Response {
    let Some(username) = req.extensions().get::<AuthenticatedUser>().and_then(AuthenticatedUser::username) else {
        return next.run(req).await;
    };
    let viewer = match access.viewer(&username).await {
        Ok(Some(viewer)) => viewer,
        Ok(None) => return next.run(req).await,
        // Never fall back to "no limit" when the lookup fails: that would show a branch user everything
        Err(e) => return unavailable(e),
    };

    // Read both before any await: the request is not `Sync`, so it cannot be borrowed across one
    let (mut company, branch) = {
        let header = |name: &str| {
            req.headers().get(name).and_then(|v| v.to_str().ok()).and_then(|v| Uuid::parse_str(v.trim()).ok())
        };
        (header(COMPANY_HEADER), header(BRANCH_HEADER))
    };
    let mut picked_branch = None;
    if !viewer.unlimited {
        // Their own reach decides what is a choice (`view_scope_for` ignores the rest)
        picked_branch = branch.map(|b| (b, Uuid::nil()));
    } else {
        // Anything outside the organization is not a choice: fall back to what is left
        if let Some(c) = company
            && !access.company_in_organization(viewer.organization_id, c).await.unwrap_or(false)
        {
            company = None;
        }
        if let Some(b) = branch
            && let Ok(Some(branch_company)) = access.branch_company(viewer.organization_id, b).await
        {
            picked_branch = Some((b, branch_company));
        }
    }

    // A screen shows data only from where the account holds ITS permission: the groups that grant
    // READ on the program of the path decide what counts (other paths, and writes, use everything
    // the account holds)
    // (a nested router sees the path without its prefix: the original URI has the whole path)
    let full_path = req
        .extensions()
        .get::<axum::extract::OriginalUri>()
        .map(|u| u.0.path().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());
    let groups = if matches!(req.method().as_str(), "GET" | "HEAD") {
        match derive_audit_target(req.method().as_str(), &full_path).0 {
            Some(program) => match access.granting_groups(&username, &program, PermissionAction::Read).await {
                // No group of its own (the page may be granted through another program): no narrowing
                Ok(Some(groups)) if !groups.is_empty() => Some(groups),
                Ok(_) => None,
                Err(e) => return unavailable(e),
            },
            None => None,
        }
    } else {
        None
    };
    let Some(scope) = view_scope_for(&viewer, groups.as_deref(), company, picked_branch) else {
        // The company / branch picked in the switcher is outside where the account holds this permission
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "success": false,
                "code": "FORBIDDEN",
                "message": "No permission for this screen in the selected company / branch",
            })),
        )
            .into_response();
    };
    application_core::view_scope::scope(scope, next.run(req)).await
}

/// A failed lookup is an error, never an unlimited view: that would show a branch user everything
fn unavailable(e: impl std::fmt::Display) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({ "success": false, "code": "DATABASE_ERROR", "message": e.to_string() })),
    )
        .into_response()
}
