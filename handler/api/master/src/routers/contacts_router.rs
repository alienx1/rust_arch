//! Contacts of organizations, companies, branches and employees (`GET` / `PUT .../{id}/contacts`).
//!
//! The routes are merged into the router of the owner's program, so the path keeps the program
//! segment the audit trail and the cache invalidation look for: `/db/dbpg001/organizations/{id}/contacts`,
//! `/db/dbpg001/companies/{id}/contacts`, `/db/dbpg002/branches/{id}/contacts`,
//! `/db/dbpg003/employees/{id}/contacts`. Reading needs `READ` on the program, replacing the list
//! `WRITE` (employees: `WRITE` in the department the employee works in, like any other change). A
//! record outside what the request may look at (`application_core::view_scope`) is a 404.

use application_master::features::db::contacts::{ContactCommands, ContactError, ContactInput, ContactOwner};
use application_master::features::db::{dbpg001::Dbpg001Commands, dbpg002::Dbpg002Commands};
use application_master::features::db::dbpg003::Dbpg003Commands;
use application_master::features::su::access_control::{AccessControl, PermissionAction};
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use domain::entity::sea_orm_active_enums::ScopeLevel;
use infrastructure::middleware::AuthenticatedUser;
use infrastructure::wire::{WireBody, WireFormat};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use super::access::{require, require_unit};
use crate::connection::AppConnection;

#[derive(Serialize)]
struct ContactErrorResponse {
    success: bool,
    code: &'static str,
    message: String,
}

/// HTTP adapter for `ContactError`
pub struct ContactApiError(pub ContactError);

impl From<ContactError> for ContactApiError {
    fn from(err: ContactError) -> Self {
        Self(err)
    }
}

impl IntoResponse for ContactApiError {
    fn into_response(self) -> Response {
        let (status, code) = match &self.0 {
            ContactError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            ContactError::Validation(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),
            ContactError::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT"),
            ContactError::Identity(_) => (StatusCode::BAD_GATEWAY, "IDENTITY_ERROR"),
            ContactError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
        };
        (status, Json(ContactErrorResponse { success: false, code, message: self.0.to_string() })).into_response()
    }
}

fn not_found(id: Uuid) -> ContactApiError {
    ContactError::NotFound(format!("{id} not found")).into()
}

/// The record must exist and be inside what the request may look at
async fn visible(owner: ContactOwner, id: Uuid, app: &AppConnection) -> Result<(), ContactApiError> {
    let db = |e: &dyn std::fmt::Display| ContactApiError(ContactError::Database(e.to_string()));
    let found = match owner {
        ContactOwner::Organization => app.service::<dyn Dbpg001Commands>().get_organization_by_id(id).await.map_err(|e| db(&e))?.is_some(),
        ContactOwner::Company => app.service::<dyn Dbpg001Commands>().get_company_by_id(id).await.map_err(|e| db(&e))?.is_some(),
        ContactOwner::Branch => app.service::<dyn Dbpg002Commands>().get_branch_by_id(id).await.map_err(|e| db(&e))?.is_some(),
        ContactOwner::Employee => app.service::<dyn Dbpg003Commands>().get_employee_by_id(id).await.map_err(|e| db(&e))?.is_some(),
    };
    if found { Ok(()) } else { Err(not_found(id)) }
}

/// Department an employee works in now (the employee id itself when unknown: matches no unit and
/// the visibility check answers 404)
async fn employee_department(commands: &Arc<dyn Dbpg003Commands>, id: Uuid) -> Uuid {
    match commands.get_employee_by_id(id).await {
        Ok(Some(e)) => e.department_id,
        _ => id,
    }
}

async fn list_for(
    owner: ContactOwner,
    program: &'static str,
    State(app): State<AppConnection>,
    user: AuthenticatedUser,
    wire: WireFormat,
    id: Uuid,
) -> Result<Response, ContactApiError> {
    let access: Arc<dyn AccessControl> = app.service::<dyn AccessControl>();
    if let Err(denied) = require(&access, &user, program, PermissionAction::Read).await {
        return Ok(denied);
    }
    visible(owner, id, &app).await?;
    let contacts: Arc<dyn ContactCommands> = app.service::<dyn ContactCommands>();
    Ok(wire.respond(contacts.list_contacts(owner, id).await?))
}

async fn replace_for(
    owner: ContactOwner,
    program: &'static str,
    State(app): State<AppConnection>,
    user: AuthenticatedUser,
    wire: WireFormat,
    id: Uuid,
    body: Vec<ContactInput>,
) -> Result<Response, ContactApiError> {
    let access: Arc<dyn AccessControl> = app.service::<dyn AccessControl>();
    let actor = if owner == ContactOwner::Employee {
        let department = employee_department(&app.service::<dyn Dbpg003Commands>(), id).await;
        require_unit(&access, &user, program, PermissionAction::Write, ScopeLevel::Department, department).await
    } else {
        require(&access, &user, program, PermissionAction::Write).await
    };
    let actor = match actor {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    visible(owner, id, &app).await?;
    let contacts: Arc<dyn ContactCommands> = app.service::<dyn ContactCommands>();
    Ok(wire.respond(contacts.replace_contacts(owner, id, body, Some(actor.user_id)).await?))
}

macro_rules! contact_handlers {
    ($get:ident, $put:ident, $owner:expr, $program:expr) => {
        pub async fn $get(
            state: State<AppConnection>,
            Extension(user): Extension<AuthenticatedUser>,
            wire: WireFormat,
            Path(id): Path<Uuid>,
        ) -> Result<Response, ContactApiError> {
            list_for($owner, $program, state, user, wire, id).await
        }

        pub async fn $put(
            state: State<AppConnection>,
            Extension(user): Extension<AuthenticatedUser>,
            wire: WireFormat,
            Path(id): Path<Uuid>,
            WireBody(body): WireBody<Vec<ContactInput>>,
        ) -> Result<Response, ContactApiError> {
            replace_for($owner, $program, state, user, wire, id, body).await
        }
    };
}

contact_handlers!(get_organization_contacts, put_organization_contacts, ContactOwner::Organization, "DBPG001");
contact_handlers!(get_company_contacts, put_company_contacts, ContactOwner::Company, "DBPG001");
contact_handlers!(get_branch_contacts, put_branch_contacts, ContactOwner::Branch, "DBPG002");
contact_handlers!(get_employee_contacts, put_employee_contacts, ContactOwner::Employee, "DBPG003");

/// Contact routes of DBPG001 (organizations and companies)
pub fn dbpg001_contact_routes() -> Router<AppConnection> {
    Router::new()
        .route("/organizations/{id}/contacts", get(get_organization_contacts).put(put_organization_contacts))
        .route("/companies/{id}/contacts", get(get_company_contacts).put(put_company_contacts))
}

/// Contact routes of DBPG002 (branches)
pub fn dbpg002_contact_routes() -> Router<AppConnection> {
    Router::new().route("/branches/{id}/contacts", get(get_branch_contacts).put(put_branch_contacts))
}

/// Contact routes of DBPG003 (employees)
pub fn dbpg003_contact_routes() -> Router<AppConnection> {
    Router::new().route("/employees/{id}/contacts", get(get_employee_contacts).put(put_employee_contacts))
}
