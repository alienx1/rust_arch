//! DBPG002 router: Branches & Departments (list + create/update).
//!
//! Mounted at `/api/v1/db/dbpg002` (path = `db_program.path` of program `DBPG002`).

use application_master::features::su::access_control::{AccessControl, PermissionAction};
use application_master::features::db::dbpg002::{
    BranchFilterQuery, CreateBranchCommand, CreateDepartmentCommand, Dbpg002Commands,
    Dbpg002Error, DepartmentFilterQuery, UpdateBranchCommand, UpdateDepartmentCommand,
};
use axum::{
    Extension, Json, Router,
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use infrastructure::middleware::AuthenticatedUser;
use infrastructure::wire::{WireBody, WireFormat};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use super::access::{require, require_unit};
use domain::entity::sea_orm_active_enums::ScopeLevel;
use crate::connection::AppConnection;

/// Path of program `DBPG002` below the API prefix.
pub const DBPG002_PATH: &str = "/db/dbpg002";
const PROGRAM: &str = "DBPG002";

/// State of the standalone router used by tests (the app router takes both from `AppConnection`)
#[derive(Clone)]
pub struct Dbpg002State {
    pub commands: Arc<dyn Dbpg002Commands>,
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<Dbpg002State> for Arc<dyn Dbpg002Commands> {
    fn from_ref(s: &Dbpg002State) -> Self {
        s.commands.clone()
    }
}

impl FromRef<Dbpg002State> for Arc<dyn AccessControl> {
    fn from_ref(s: &Dbpg002State) -> Self {
        s.access.clone()
    }
}

/// Standardized error response body for DBPG002 endpoints.
#[derive(Serialize)]
pub struct Dbpg002ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `Dbpg002Error`.
pub struct Dbpg002ApiError(pub Dbpg002Error);

impl From<Dbpg002Error> for Dbpg002ApiError {
    fn from(err: Dbpg002Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for Dbpg002ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            Dbpg002Error::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            Dbpg002Error::DuplicateCode(msg) => {
                (StatusCode::CONFLICT, "DUPLICATE_CODE", msg.clone())
            }
            Dbpg002Error::VersionConflict => (
                StatusCode::PRECONDITION_FAILED,
                "VERSION_CONFLICT",
                "Data has been modified by another transaction. Please reload and try again."
                    .to_string(),
            ),
            Dbpg002Error::InUse(msg) => (StatusCode::CONFLICT, "IN_USE", msg.clone()),
            Dbpg002Error::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone())
            }
            Dbpg002Error::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                format!("Database error: {msg}"),
            ),
            Dbpg002Error::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone())
            }
        };

        let body = Json(Dbpg002ErrorResponse {
            success: false,
            code: code.to_string(),
            message: msg,
        });
        (status, body).into_response()
    }
}

/// Companies for the company filter / parent selector
pub async fn list_company_options_handler(
    State(commands): State<Arc<dyn Dbpg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg002ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_company_options().await?))
}

/// KPI summary of branches and departments
pub async fn get_summary_handler(
    State(commands): State<Arc<dyn Dbpg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg002ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.get_summary().await?))
}

/// Active permission groups, for a department's default group. Naming the groups is for those who
/// may assign them (SUPG004), so the list needs READ there as well as on this program.
pub async fn list_permission_group_options_handler(
    State(commands): State<Arc<dyn Dbpg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg002ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    if let Err(denied) = require(&access, &user, "SUPG004", PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_permission_group_options().await?))
}

/// Branch types (HEAD / BRANCH)
pub async fn list_branch_types_handler(
    State(commands): State<Arc<dyn Dbpg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg002ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_branch_types().await?))
}

/// List branches with filters
pub async fn list_branches_handler(
    State(commands): State<Arc<dyn Dbpg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<BranchFilterQuery>,
) -> Result<Response, Dbpg002ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_branches(filter).await?))
}

/// Get branch by ID
pub async fn get_branch_handler(
    State(commands): State<Arc<dyn Dbpg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg002ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let branch = commands
        .get_branch_by_id(id)
        .await?
        .ok_or_else(|| Dbpg002Error::NotFound(format!("Branch {id} not found")))?;
    Ok(wire.respond(branch))
}

/// Create branch
pub async fn create_branch_handler(
    State(commands): State<Arc<dyn Dbpg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<CreateBranchCommand>,
) -> Result<Response, Dbpg002ApiError> {
    let actor = match require_unit(&access, &user, PROGRAM, PermissionAction::Write, ScopeLevel::Company, cmd.company_id).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    Ok(wire.respond(commands.create_branch(cmd, Some(actor.user_id)).await?))
}

/// Update branch
pub async fn update_branch_handler(
    State(commands): State<Arc<dyn Dbpg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
    WireBody(cmd): WireBody<UpdateBranchCommand>,
) -> Result<Response, Dbpg002ApiError> {
    let actor = match require_unit(&access, &user, PROGRAM, PermissionAction::Write, ScopeLevel::Branch, id).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    Ok(wire.respond(commands.update_branch(id, cmd, Some(actor.user_id)).await?))
}

/// Delete branch (owned labels/groups go with it; 409 when still referenced)
pub async fn delete_branch_handler(
    State(commands): State<Arc<dyn Dbpg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg002ApiError> {
    if let Err(denied) = require_unit(&access, &user, PROGRAM, PermissionAction::Approve, ScopeLevel::Branch, id).await {
        return Ok(denied);
    }
    commands.delete_branch(id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// List departments with filters
pub async fn list_departments_handler(
    State(commands): State<Arc<dyn Dbpg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<DepartmentFilterQuery>,
) -> Result<Response, Dbpg002ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_departments(filter).await?))
}

/// Get department by ID
pub async fn get_department_handler(
    State(commands): State<Arc<dyn Dbpg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg002ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let department = commands
        .get_department_by_id(id)
        .await?
        .ok_or_else(|| Dbpg002Error::NotFound(format!("Department {id} not found")))?;
    Ok(wire.respond(department))
}

/// Create department
pub async fn create_department_handler(
    State(commands): State<Arc<dyn Dbpg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<CreateDepartmentCommand>,
) -> Result<Response, Dbpg002ApiError> {
    let actor = match require_unit(&access, &user, PROGRAM, PermissionAction::Write, ScopeLevel::Branch, cmd.branch_id).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    // A default permission group is given to the department's accounts: that is assigning a group
    // to accounts, which needs the same right as on the user accounts page (SUPG004 WRITE)
    if cmd.default_permission_group_id.is_some()
        && let Err(denied) = require(&access, &user, "SUPG004", PermissionAction::Write).await
    {
        return Ok(denied);
    }
    Ok(wire.respond(commands.create_department(cmd, Some(actor.user_id)).await?))
}

/// Update department
pub async fn update_department_handler(
    State(commands): State<Arc<dyn Dbpg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
    WireBody(cmd): WireBody<UpdateDepartmentCommand>,
) -> Result<Response, Dbpg002ApiError> {
    let actor = match require_unit(&access, &user, PROGRAM, PermissionAction::Write, ScopeLevel::Department, id).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    // Moving to another branch needs the right there too
    if let Err(denied) = require_unit(&access, &user, PROGRAM, PermissionAction::Write, ScopeLevel::Branch, cmd.branch_id).await {
        return Ok(denied);
    }
    // Setting or changing the default permission group grants it to the department's accounts, so
    // it needs the right to assign groups (SUPG004 WRITE); leaving it as it is, or clearing it,
    // grants nothing and stays a DBPG002 matter
    if let Some(new_default) = cmd.default_permission_group_id {
        let current = commands.get_department_by_id(id).await?.and_then(|d| d.default_permission_group_id);
        if current != Some(new_default)
            && let Err(denied) = require(&access, &user, "SUPG004", PermissionAction::Write).await
        {
            return Ok(denied);
        }
    }
    Ok(wire.respond(commands.update_department(id, cmd, Some(actor.user_id)).await?))
}

/// Delete department (owned labels go with it; 409 when still referenced)
pub async fn delete_department_handler(
    State(commands): State<Arc<dyn Dbpg002Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg002ApiError> {
    if let Err(denied) = require_unit(&access, &user, PROGRAM, PermissionAction::Approve, ScopeLevel::Department, id).await {
        return Ok(denied);
    }
    commands.delete_department(id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Sub-router for DBPG002 endpoints mounted into `AppConnection`.
pub fn dbpg002_routes() -> Router<AppConnection> {
    Router::new()
        .merge(super::contacts_router::dbpg002_contact_routes())
        .route("/companies", get(list_company_options_handler))
        .route("/summary", get(get_summary_handler))
        .route("/branches/types", get(list_branch_types_handler))
        .route("/options/permission-groups", get(list_permission_group_options_handler))
        .route("/branches", get(list_branches_handler).post(create_branch_handler))
        .route(
            "/branches/{id}",
            get(get_branch_handler)
                .put(update_branch_handler)
                .delete(delete_branch_handler),
        )
        .route("/departments", get(list_departments_handler).post(create_department_handler))
        .route(
            "/departments/{id}",
            get(get_department_handler)
                .put(update_department_handler)
                .delete(delete_department_handler),
        )
}

/// Same routes with explicit state (useful for tests).
pub fn dbpg002_routes_with_state(state: Dbpg002State) -> Router {
    Router::new()
        .route("/companies", get(list_company_options_handler))
        .route("/summary", get(get_summary_handler))
        .route("/branches/types", get(list_branch_types_handler))
        .route("/options/permission-groups", get(list_permission_group_options_handler))
        .route("/branches", get(list_branches_handler).post(create_branch_handler))
        .route(
            "/branches/{id}",
            get(get_branch_handler)
                .put(update_branch_handler)
                .delete(delete_branch_handler),
        )
        .route("/departments", get(list_departments_handler).post(create_department_handler))
        .route(
            "/departments/{id}",
            get(get_department_handler)
                .put(update_department_handler)
                .delete(delete_department_handler),
        )
        .with_state(state)
}
