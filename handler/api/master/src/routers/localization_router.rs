//! Localization router for frontend i18n translation dictionary lookups.
//!
//! # Scope & Responsibilities
//! - Exclusively serves frontend i18n UI dictionaries.
//! - Calls the `LocalizationCommands` trait interface.
//! - Accepts language ID (`lid` of `db_language`).
//! - Returns `I18nDictionary` with `<category>.<key> => <value>` mapping.

use application_master::features::localization::{
    LocalizationCommands, LocalizationError,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::connection::AppConnection;

/// Query parameters for i18n translation dictionary lookup.
#[derive(Debug, Deserialize)]
pub struct I18nQueryParams {
    /// Language ID (`db_language.id` / lid)
    pub lid: Option<Uuid>,
    /// Language ID alias (`language_id`)
    pub language_id: Option<Uuid>,
}

/// Standardized error response body for localization endpoints.
#[derive(Serialize)]
pub struct LocalizationErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// HTTP API error adapter for `LocalizationError`.
pub struct LocalizationApiError(pub LocalizationError);

impl From<LocalizationError> for LocalizationApiError {
    fn from(err: LocalizationError) -> Self {
        Self(err)
    }
}

impl IntoResponse for LocalizationApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self.0 {
            LocalizationError::LanguageNotFound(msg) => {
                (StatusCode::NOT_FOUND, "LANGUAGE_NOT_FOUND", msg.clone())
            }
            LocalizationError::LabelNotFound(msg) => {
                (StatusCode::NOT_FOUND, "LABEL_NOT_FOUND", msg.clone())
            }
            LocalizationError::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone())
            }
            LocalizationError::Database(msg) => {
                tracing::error!("Database error in localization: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "A database error occurred while fetching translations".to_string(),
                )
            }
        };

        let body = Json(LocalizationErrorResponse {
            success: false,
            code: code.to_string(),
            message: msg,
        });

        (status, body).into_response()
    }
}

use infrastructure::wire::WireFormat;

/// Handler for retrieving i18n dictionary by path parameter.
///
/// Example: `GET /api/v1/master/localization/{lid}`
pub async fn get_i18n_by_path(
    State(commands): State<Arc<dyn LocalizationCommands>>,
    wire: WireFormat,
    Path(lid): Path<Uuid>,
) -> Result<Response, LocalizationApiError> {
    let dict = commands.get_i18n(lid).await?;
    Ok(wire.respond(dict))
}

/// Handler for retrieving i18n dictionary by query parameter.
///
/// Example: `GET /api/v1/master/localization?lid=<uuid>`
pub async fn get_i18n_by_query(
    State(commands): State<Arc<dyn LocalizationCommands>>,
    wire: WireFormat,
    Query(params): Query<I18nQueryParams>,
) -> Result<Response, LocalizationApiError> {
    let lid = params.lid.or(params.language_id).ok_or_else(|| {
        LocalizationApiError(LocalizationError::Validation(
            "Missing query parameter 'lid' or 'language_id'".to_string(),
        ))
    })?;

    let dict = commands.get_i18n(lid).await?;
    Ok(wire.respond(dict))
}

/// Handler for retrieving i18n dictionary by language code (`th`, `en`; case-insensitive).
///
/// Example: `GET /api/v1/i18n/code/th`
pub async fn get_i18n_by_code(
    State(commands): State<Arc<dyn LocalizationCommands>>,
    wire: WireFormat,
    Path(code): Path<String>,
) -> Result<Response, LocalizationApiError> {
    let dict = commands.get_i18n_by_code(&code).await?;
    Ok(wire.respond(dict))
}

/// Builds the localization sub-router.
///
/// Routes provided:
/// - `GET /` with query `?lid=<uuid>` or `?language_id=<uuid>`
/// - `GET /{lid}` with path parameter `<uuid>`
/// - `GET /i18n/{lid}` alias with path parameter `<uuid>`
/// - `GET /code/{code}` with a language code (`th`, `en`)
pub fn localization_routes() -> Router<AppConnection> {
    Router::new()
        .route("/", get(get_i18n_by_query))
        .route("/{lid}", get(get_i18n_by_path))
        .route("/i18n/{lid}", get(get_i18n_by_path))
        .route("/code/{code}", get(get_i18n_by_code))
}

/// Builds the localization sub-router with explicit `Arc<dyn LocalizationCommands>` state.
/// Useful for modular composition and isolated unit testing.
#[allow(dead_code)]
pub fn localization_routes_with_commands(commands: Arc<dyn LocalizationCommands>) -> Router {
    Router::new()
        .route("/", get(get_i18n_by_query))
        .route("/{lid}", get(get_i18n_by_path))
        .route("/i18n/{lid}", get(get_i18n_by_path))
        .route("/code/{code}", get(get_i18n_by_code))
        .with_state(commands)
}
