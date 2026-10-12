//! DBPG003 router: Employees (paged list + create/update).
//!
//! Mounted at `/api/v1/db/dbpg003` (path = `db_program.path` of program `DBPG003`).

use application_master::features::su::access_control::{AccessControl, PermissionAction};
use application_master::features::db::dbpg003::{
    CreateEmployeeCommand, Dbpg003Commands, Dbpg003Error, EmployeeFilterQuery,
    UpdateEmployeeCommand,
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

/// Path of program `DBPG003` below the API prefix.
pub const DBPG003_PATH: &str = "/db/dbpg003";
const PROGRAM: &str = "DBPG003";

/// State of the standalone router used by tests (the app router takes both from `AppConnection`)
#[derive(Clone)]
pub struct Dbpg003State {
    pub commands: Arc<dyn Dbpg003Commands>,
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<Dbpg003State> for Arc<dyn Dbpg003Commands> {
    fn from_ref(s: &Dbpg003State) -> Self {
        s.commands.clone()
    }
}

impl FromRef<Dbpg003State> for Arc<dyn AccessControl> {
    fn from_ref(s: &Dbpg003State) -> Self {
        s.access.clone()
    }
}

/// Standardized error response body for DBPG003 endpoints.
#[derive(Serialize)]
pub struct Dbpg003ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `Dbpg003Error`.
pub struct Dbpg003ApiError(pub Dbpg003Error);

impl From<Dbpg003Error> for Dbpg003ApiError {
    fn from(err: Dbpg003Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for Dbpg003ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            Dbpg003Error::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            Dbpg003Error::DuplicateCode(msg) => {
                (StatusCode::CONFLICT, "DUPLICATE_CODE", msg.clone())
            }
            Dbpg003Error::VersionConflict => (
                StatusCode::PRECONDITION_FAILED,
                "VERSION_CONFLICT",
                "Data has been modified by another transaction. Please reload and try again."
                    .to_string(),
            ),
            Dbpg003Error::InUse(msg) => (StatusCode::CONFLICT, "IN_USE", msg.clone()),
            Dbpg003Error::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone())
            }
            Dbpg003Error::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                format!("Database error: {msg}"),
            ),
            Dbpg003Error::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone())
            }
        };

        let body = Json(Dbpg003ErrorResponse {
            success: false,
            code: code.to_string(),
            message: msg,
        });
        (status, body).into_response()
    }
}

/// Companies for the selectors / filter
pub async fn list_company_options_handler(
    State(commands): State<Arc<dyn Dbpg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg003ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_company_options().await?))
}

/// Branches for the cascading selectors
pub async fn list_branch_options_handler(
    State(commands): State<Arc<dyn Dbpg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg003ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_branch_options().await?))
}

/// Departments for the cascading selectors
pub async fn list_department_options_handler(
    State(commands): State<Arc<dyn Dbpg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg003ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_department_options().await?))
}

/// Name prefixes
pub async fn list_prefixes_handler(
    State(commands): State<Arc<dyn Dbpg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg003ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_prefixes().await?))
}

/// KPI summary
pub async fn get_summary_handler(
    State(commands): State<Arc<dyn Dbpg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg003ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.get_summary().await?))
}

/// One page of employees (`page`, `limit`, `search`, `active`, `company_id`, `branch_id`, `department_id`)
pub async fn list_employees_handler(
    State(commands): State<Arc<dyn Dbpg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<EmployeeFilterQuery>,
) -> Result<Response, Dbpg003ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_employees(filter).await?))
}

/// Get employee by ID
pub async fn get_employee_handler(
    State(commands): State<Arc<dyn Dbpg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg003ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let employee = commands
        .get_employee_by_id(id)
        .await?
        .ok_or_else(|| Dbpg003Error::NotFound(format!("Employee {id} not found")))?;
    Ok(wire.respond(employee))
}

/// Department an employee works in now, so a change is checked against where the record lives.
/// An unknown employee yields the employee id itself, which matches no unit: the command then
/// answers 404.
async fn current_department(commands: &Arc<dyn Dbpg003Commands>, employee_id: Uuid) -> Uuid {
    match commands.get_employee_by_id(employee_id).await {
        Ok(Some(employee)) => employee.department_id,
        _ => employee_id,
    }
}

/// Create employee
pub async fn create_employee_handler(
    State(commands): State<Arc<dyn Dbpg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<CreateEmployeeCommand>,
) -> Result<Response, Dbpg003ApiError> {
    let actor = match require_unit(&access, &user, PROGRAM, PermissionAction::Write, ScopeLevel::Department, cmd.department_id).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    Ok(wire.respond(commands.create_employee(cmd, Some(actor.user_id)).await?))
}

/// Update employee
pub async fn update_employee_handler(
    State(commands): State<Arc<dyn Dbpg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<Uuid>,
    WireBody(cmd): WireBody<UpdateEmployeeCommand>,
) -> Result<Response, Dbpg003ApiError> {
    let actor = match require_unit(&access, &user, PROGRAM, PermissionAction::Write, ScopeLevel::Department, current_department(&commands, id).await).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    // Moving to another department needs the right there too
    if let Err(denied) = require_unit(&access, &user, PROGRAM, PermissionAction::Write, ScopeLevel::Department, cmd.department_id).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.update_employee(id, cmd, Some(actor.user_id)).await?))
}

/// Delete employee (hard delete; 409 `IN_USE` while still referenced)
pub async fn delete_employee_handler(
    State(commands): State<Arc<dyn Dbpg003Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Response, Dbpg003ApiError> {
    if let Err(denied) = require_unit(&access, &user, PROGRAM, PermissionAction::Approve, ScopeLevel::Department, current_department(&commands, id).await).await {
        return Ok(denied);
    }
    commands.delete_employee(id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Sub-router for DBPG003 endpoints mounted into `AppConnection`.
pub fn dbpg003_routes() -> Router<AppConnection> {
    Router::new()
        .route("/companies", get(list_company_options_handler))
        .route("/branches", get(list_branch_options_handler))
        .route("/departments", get(list_department_options_handler))
        .route("/prefixes", get(list_prefixes_handler))
        .route("/summary", get(get_summary_handler))
        .route("/employees", get(list_employees_handler).post(create_employee_handler))
        .route("/employees/{id}", get(get_employee_handler).put(update_employee_handler).delete(delete_employee_handler))
}

/// Same routes with explicit state (useful for tests).
pub fn dbpg003_routes_with_state(state: Dbpg003State) -> Router {
    Router::new()
        .route("/companies", get(list_company_options_handler))
        .route("/branches", get(list_branch_options_handler))
        .route("/departments", get(list_department_options_handler))
        .route("/prefixes", get(list_prefixes_handler))
        .route("/summary", get(get_summary_handler))
        .route("/employees", get(list_employees_handler).post(create_employee_handler))
        .route("/employees/{id}", get(get_employee_handler).put(update_employee_handler).delete(delete_employee_handler))
        .with_state(state)
}
