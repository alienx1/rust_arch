//! Authorization helper shared by the security-administration routers (SUPG001–SUPG004).

use application_master::features::su::access_control::{AccessControl, Actor, PermissionAction, ScopeTarget};
use domain::entity::sea_orm_active_enums::ScopeLevel;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use infrastructure::middleware::AuthenticatedUser;
use serde_json::json;
use std::sync::Arc;

use super::username_of;

/// 403 body in the same shape as the feature error responses
fn forbidden(program_code: &str, action: PermissionAction) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "success": false,
            "code": "FORBIDDEN",
            "message": format!("{} permission on {program_code} is required", action.code()),
        })),
    )
        .into_response()
}

/// The acting user when the signed-in account may perform `action` on `program_code`, otherwise
/// the ready-to-return 403 (or 500 when the permission lookup itself fails).
pub async fn require(
    access: &Arc<dyn AccessControl>,
    user: &AuthenticatedUser,
    program_code: &str,
    action: PermissionAction,
) -> Result<Actor, Response> {
    require_in(access, user, program_code, action, None).await
}

fn database_error(e: impl std::fmt::Display) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "success": false, "code": "DATABASE_ERROR", "message": e.to_string() })),
    )
        .into_response()
}

/// Like [`require`], but for work on one unit: grants limited to a scope of the user's group only
/// count when `target` is inside that scope (`None` = anywhere).
pub async fn require_in(
    access: &Arc<dyn AccessControl>,
    user: &AuthenticatedUser,
    program_code: &str,
    action: PermissionAction,
    target: Option<ScopeTarget>,
) -> Result<Actor, Response> {
    let Some(username) = username_of(user) else {
        return Err(forbidden(program_code, action));
    };
    match access.authorize_in(&username, program_code, action, target).await {
        Ok(Some(actor)) => Ok(actor),
        Ok(None) => Err(forbidden(program_code, action)),
        Err(e) => Err(database_error(e)),
    }
}

/// [`require_in`] for a unit given by id (an organization / company / branch / department row):
/// the unit's chain of parents is resolved first. An id that matches no unit is checked as
/// "anywhere" and left to the command to report as not found.
pub async fn require_unit(
    access: &Arc<dyn AccessControl>,
    user: &AuthenticatedUser,
    program_code: &str,
    action: PermissionAction,
    level: ScopeLevel,
    unit_id: uuid::Uuid,
) -> Result<Actor, Response> {
    let target = access.unit_target(level, unit_id).await.map_err(database_error)?;
    require_in(access, user, program_code, action, target).await
}

/// Like [`require`], but the action may be held on ANY of `program_codes` (the first one that
/// grants it wins). Used where one page is reached through a different catalogue program than
/// the API path it calls (the organization list lives at `/db/dbpg000` but is the page of the
/// menu entry `DBPG001`).
pub async fn require_any(
    access: &Arc<dyn AccessControl>,
    user: &AuthenticatedUser,
    program_codes: &[&str],
    action: PermissionAction,
) -> Result<Actor, Response> {
    let mut last = None;
    for code in program_codes {
        match require(access, user, code, action).await {
            Ok(actor) => return Ok(actor),
            Err(denied) => last = Some(denied),
        }
    }
    Err(last.unwrap_or_else(|| forbidden(program_codes.first().copied().unwrap_or(""), action)))
}
