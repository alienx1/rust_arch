//! RPPG000 router: the signed-in user's report jobs and a preview and download of each file (the
//! "report is ready" push goes out through the notification service, not from here).
//!
//! Mounted at `/api/v1/reports` behind a Kratos session. Everyone sees only their own jobs (a
//! job of somebody else is a 404); no program permission is needed to read your own results.

use application_report::features::rppg000::{Rppg000Commands, Rppg000Error};
use axum::{
    Extension, Json, Router,
    extract::{FromRef, Path, Query, State},
    http::{StatusCode, header},
    response::{
        IntoResponse, Response,
    },
    routing::get,
};
use infrastructure::middleware::AuthenticatedUser;
use infrastructure::wire::WireFormat;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::connection::ReportConnection;

/// Path of program `RPPG000` below the API prefix (the prefix itself is `/api/v1/reports`)
pub const RPPG000_PATH: &str = "";

impl FromRef<ReportConnection> for Arc<dyn Rppg000Commands> {
    fn from_ref(c: &ReportConnection) -> Self {
        c.rppg000_commands.clone()
    }
}

#[derive(Serialize)]
pub struct Rppg000ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

pub struct Rppg000ApiError(pub Rppg000Error);

impl From<Rppg000Error> for Rppg000ApiError {
    fn from(e: Rppg000Error) -> Self {
        Self(e)
    }
}

impl IntoResponse for Rppg000ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self.0 {
            Rppg000Error::Validation(m) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", m.clone()),
            Rppg000Error::NotFound(m) => (StatusCode::NOT_FOUND, "NOT_FOUND", m.clone()),
            Rppg000Error::NotReady(m) => (StatusCode::CONFLICT, "REPORT_NOT_READY", m.clone()),
            Rppg000Error::FileGone(m) => (StatusCode::GONE, "REPORT_FILE_GONE", m.clone()),
            Rppg000Error::Storage(m) => {
                (StatusCode::SERVICE_UNAVAILABLE, "STORAGE_UNAVAILABLE", format!("File storage unavailable: {m}"))
            }
            Rppg000Error::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", m.clone()),
        };
        (status, Json(Rppg000ErrorResponse { success: false, code: code.to_string(), message })).into_response()
    }
}

fn username(user: &AuthenticatedUser) -> Result<String, Rppg000ApiError> {
    user.username().ok_or_else(|| Rppg000ApiError(Rppg000Error::Validation("The session has no username".into())))
}

/// `GET /jobs`: my reports, newest first
pub async fn list_jobs_handler(
    State(commands): State<Arc<dyn Rppg000Commands>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
) -> Result<Response, Rppg000ApiError> {
    Ok(wire.respond(commands.list_jobs(&username(&user)?).await?))
}

/// `GET /jobs/{id}`
pub async fn get_job_handler(
    State(commands): State<Arc<dyn Rppg000Commands>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<String>,
) -> Result<Response, Rppg000ApiError> {
    Ok(wire.respond(commands.get_job(&username(&user)?, &id).await?))
}

#[derive(Debug, Deserialize)]
pub struct PreviewQuery {
    pub rows: Option<usize>,
}

/// `GET /jobs/{id}/preview?rows=`: the first rows of the file
pub async fn preview_job_handler(
    State(commands): State<Arc<dyn Rppg000Commands>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<String>,
    Query(q): Query<PreviewQuery>,
) -> Result<Response, Rppg000ApiError> {
    Ok(wire.respond(commands.preview_job(&username(&user)?, &id, q.rows).await?))
}

/// Answer of `GET /jobs/{id}/download`
#[derive(Serialize)]
pub struct DownloadLink {
    /// Short-lived signed storage URL that saves the file
    pub url: String,
}

/// `GET /jobs/{id}/download`: a short-lived signed link to the file. It is returned as data, not
/// as a redirect: a reverse proxy that follows redirects itself (the Trunk dev server does) would
/// call the storage with its own `Host` header, which breaks the signature. The web app opens the
/// link in the browser.
pub async fn download_job_handler(
    State(commands): State<Arc<dyn Rppg000Commands>>,
    Extension(user): Extension<AuthenticatedUser>,
    wire: WireFormat,
    Path(id): Path<String>,
) -> Result<Response, Rppg000ApiError> {
    let url = commands.download_url(&username(&user)?, &id).await?;
    let mut response = wire.respond(DownloadLink { url });
    response.headers_mut().insert(header::CACHE_CONTROL, header::HeaderValue::from_static("no-store"));
    Ok(response)
}

pub fn rppg000_routes() -> Router<ReportConnection> {
    Router::new()
        .route("/jobs", get(list_jobs_handler))
        .route("/jobs/{id}", get(get_job_handler))
        .route("/jobs/{id}/preview", get(preview_job_handler))
        .route("/jobs/{id}/download", get(download_job_handler))
}
