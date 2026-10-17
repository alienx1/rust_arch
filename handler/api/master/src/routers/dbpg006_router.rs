//! DBPG006 router: i18n Labels (`db_label`): list/search, create, edit and delete label entries.
//!
//! Mounted at `/api/v1/db/dbpg006` behind a Kratos session. Reading needs READ and writing needs
//! WRITE and deleting needs APPROVE on program `DBPG006`. An entry is identified by the query parameters `category` and
//! `key` (keys contain dots and may contain any character, so they are not path segments).

use application_master::features::db::dbpg006::{
    CreateLabelCommand, Dbpg006Commands, Dbpg006Error, LabelFilterQuery, UpdateLabelCommand,
};
use application_master::features::su::access_control::{AccessControl, PermissionAction};
use axum::{
    Extension, Json, Router,
    extract::{FromRef, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use infrastructure::middleware::AuthenticatedUser;
use infrastructure::wire::{WireBody, WireFormat};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::access::require;
use crate::connection::AppConnection;

/// Path of program `DBPG006` below the API prefix.
pub const DBPG006_PATH: &str = "/db/dbpg006";
const PROGRAM: &str = "DBPG006";

/// State of the standalone router used by tests (the app router takes both from `AppConnection`)
#[derive(Clone)]
pub struct Dbpg006State {
    pub commands: Arc<dyn Dbpg006Commands>,
    pub access: Arc<dyn AccessControl>,
}

impl FromRef<Dbpg006State> for Arc<dyn Dbpg006Commands> {
    fn from_ref(s: &Dbpg006State) -> Self {
        s.commands.clone()
    }
}

impl FromRef<Dbpg006State> for Arc<dyn AccessControl> {
    fn from_ref(s: &Dbpg006State) -> Self {
        s.access.clone()
    }
}

/// Standardized error response body for DBPG006 endpoints.
#[derive(Serialize)]
pub struct Dbpg006ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `Dbpg006Error`.
pub struct Dbpg006ApiError(pub Dbpg006Error);

impl From<Dbpg006Error> for Dbpg006ApiError {
    fn from(err: Dbpg006Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for Dbpg006ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            Dbpg006Error::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            Dbpg006Error::Duplicate(msg) => (StatusCode::CONFLICT, "DUPLICATE", msg.clone()),
            Dbpg006Error::VersionConflict => (
                StatusCode::PRECONDITION_FAILED,
                "VERSION_CONFLICT",
                "Data has been modified by another transaction. Please reload and try again.".to_string(),
            ),
            Dbpg006Error::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone())
            }
            Dbpg006Error::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                format!("Database error: {msg}"),
            ),
            Dbpg006Error::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone())
            }
        };
        let body = Json(Dbpg006ErrorResponse { success: false, code: code.to_string(), message: msg });
        (status, body).into_response()
    }
}

/// Query identifying one entry
#[derive(Debug, Deserialize)]
pub struct LabelEntryQuery {
    pub category: String,
    pub key: String,
}

/// Active languages
pub async fn list_languages_handler(
    State(commands): State<Arc<dyn Dbpg006Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg006ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_languages().await?))
}

/// Categories with entry counts
pub async fn list_categories_handler(
    State(commands): State<Arc<dyn Dbpg006Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Dbpg006ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_categories().await?))
}

/// One page of entries (`category`, `search`, `page`, `limit`)
pub async fn list_labels_handler(
    State(commands): State<Arc<dyn Dbpg006Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(filter): Query<LabelFilterQuery>,
) -> Result<Response, Dbpg006ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    Ok(wire.respond(commands.list_labels(filter).await?))
}

/// One entry (`category`, `key`)
pub async fn get_label_handler(
    State(commands): State<Arc<dyn Dbpg006Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Query(q): Query<LabelEntryQuery>,
) -> Result<Response, Dbpg006ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Read).await {
        return Ok(denied);
    }
    let entry = commands.get_label(&q.category, &q.key).await?.ok_or_else(|| {
        Dbpg006Error::NotFound(format!("Label '{}' not found in category '{}'", q.key, q.category))
    })?;
    Ok(wire.respond(entry))
}

/// Create entry
pub async fn create_label_handler(
    State(commands): State<Arc<dyn Dbpg006Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<CreateLabelCommand>,
) -> Result<Response, Dbpg006ApiError> {
    let actor = match require(&access, &user, PROGRAM, PermissionAction::Write).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    Ok(wire.respond(commands.create_label(cmd, Some(actor.user_id)).await?))
}

/// Update entry
pub async fn update_label_handler(
    State(commands): State<Arc<dyn Dbpg006Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    WireBody(cmd): WireBody<UpdateLabelCommand>,
) -> Result<Response, Dbpg006ApiError> {
    let actor = match require(&access, &user, PROGRAM, PermissionAction::Write).await {
        Ok(actor) => actor,
        Err(denied) => return Ok(denied),
    };
    Ok(wire.respond(commands.update_label(cmd, Some(actor.user_id)).await?))
}

/// Delete entry (all languages)
pub async fn delete_label_handler(
    State(commands): State<Arc<dyn Dbpg006Commands>>,
    State(access): State<Arc<dyn AccessControl>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(q): Query<LabelEntryQuery>,
) -> Result<Response, Dbpg006ApiError> {
    if let Err(denied) = require(&access, &user, PROGRAM, PermissionAction::Approve).await {
        return Ok(denied);
    }
    commands.delete_label(&q.category, &q.key).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Sub-router for DBPG006 endpoints mounted into `AppConnection`.
pub fn dbpg006_routes() -> Router<AppConnection> {
    Router::new()
        .route("/languages", get(list_languages_handler))
        .route("/categories", get(list_categories_handler))
        .route(
            "/labels",
            get(list_labels_handler)
                .post(create_label_handler)
                .put(update_label_handler)
                .delete(delete_label_handler),
        )
        .route("/labels/entry", get(get_label_handler))
}

/// Same routes with explicit state (useful for tests).
pub fn dbpg006_routes_with_state(state: Dbpg006State) -> Router {
    Router::new()
        .route("/languages", get(list_languages_handler))
        .route("/categories", get(list_categories_handler))
        .route(
            "/labels",
            get(list_labels_handler)
                .post(create_label_handler)
                .put(update_label_handler)
                .delete(delete_label_handler),
        )
        .route("/labels/entry", get(get_label_handler))
        .with_state(state)
}
