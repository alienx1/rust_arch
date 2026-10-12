//! Base router for api-master: health check, readiness, liveness, and service metadata.

use crate::connection::AppConnection;
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use infrastructure::middleware::{
    ErrorLocalizer, audit_trail, cors_layer_from_env, localize_error_responses, require_kratos_session,
    set_request_language,
};
use serde::Serialize;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

/// Health check response body.
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
    pub database: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<String>,
}

/// Service information response body.
#[derive(Serialize)]
pub struct ServiceInfoResponse {
    pub service: &'static str,
    pub description: &'static str,
    pub version: &'static str,
    pub docs: &'static str,
}

/// Liveness probe handler (Kubernetes livez).
pub async fn livez() -> impl IntoResponse {
    (StatusCode::OK, "alive")
}

/// Readiness probe handler (Kubernetes readyz).
pub async fn readyz(State(conn): State<AppConnection>) -> impl IntoResponse {
    if conn.ping_db().await.is_ok() {
        (StatusCode::OK, "ready")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "database not ready")
    }
}

/// Health check handler returning detailed subsystem status.
pub async fn health_check(State(conn): State<AppConnection>) -> impl IntoResponse {
    let db_status = match conn.ping_db().await {
        Ok(_) => "connected".to_string(),
        Err(e) => format!("unhealthy: {e}"),
    };

    let cache_status = match conn.ping_cache().await {
        Ok(_) if conn.cache.is_some() => Some("connected".to_string()),
        Ok(_) => None,
        Err(e) => Some(format!("unhealthy: {e}")),
    };

    let auth_status = conn
        .kratos
        .as_ref()
        .map(|k| format!("configured ({})", k.public_url));

    let is_healthy = db_status == "connected";
    let status_code = if is_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(HealthResponse {
            status: if is_healthy { "healthy" } else { "degraded" },
            service: "api-master",
            version: env!("CARGO_PKG_VERSION"),
            database: db_status,
            cache: cache_status,
            auth: auth_status,
        }),
    )
}

/// Service metadata / root info handler.
pub async fn service_info() -> impl IntoResponse {
    Json(ServiceInfoResponse {
        service: "api-master",
        description: "Master Data Management Service (PostgreSQL 17 / pgEdge, Valkey, Ory Kratos)",
        version: env!("CARGO_PKG_VERSION"),
        docs: "/docs",
    })
}

/// Base routes (service info, health, liveness, readiness).
pub fn base_routes() -> Router<AppConnection> {
    Router::new()
        .route("/", get(service_info))
        .route("/health", get(health_check))
        .route("/livez", get(livez))
        .route("/readyz", get(readyz))
}

/// Puts `routes` behind Ory Kratos session authentication.
///
/// Fails closed: when Kratos is not configured, every request is rejected with 503 rather than
/// being served unauthenticated.
fn protected(routes: Router<AppConnection>, conn: &AppConnection) -> Router<AppConnection> {
    match &conn.kratos {
        Some(kratos) => routes.route_layer(axum::middleware::from_fn_with_state(
            kratos.clone(),
            require_kratos_session,
        )),
        None => routes.route_layer(axum::middleware::from_fn(
            |_req: axum::extract::Request, _next: axum::middleware::Next| async {
                (StatusCode::SERVICE_UNAVAILABLE, "authentication provider not configured")
            },
        )),
    }
}

/// Build the top-level application router with CORS, tracing, and prefixing.
pub fn build_app_router(conn: AppConnection, prefix: &str) -> Router {
    let clean_prefix = if prefix.starts_with('/') {
        prefix.to_string()
    } else {
        format!("/{}", prefix)
    };

    // Sub-router mounted under prefix (e.g. /api/v1/master)
    let api_router = Router::new()
        .merge(base_routes())
        .nest(super::DBPG000_PATH, protected(super::dbpg000_routes(), &conn))
        .nest(super::DBPG001_PATH, protected(super::dbpg001_routes(), &conn))
        .nest(super::DBPG002_PATH, protected(super::dbpg002_routes(), &conn))
        .nest(super::DBPG003_PATH, protected(super::dbpg003_routes(), &conn))
        .nest(super::DBPG004_PATH, protected(super::dbpg004_routes(), &conn))
        .nest(super::DBPG005_PATH, protected(super::dbpg005_routes(), &conn))
        .nest(super::DBPG006_PATH, protected(super::dbpg006_routes(), &conn))
        .nest("/localization", super::localization_routes())
        .nest("/i18n", super::localization_routes())
        .nest("/dashboard", protected(super::dashboard_routes(), &conn))
        .nest(super::SUPG001_PATH, protected(super::supg001_routes(), &conn))
        .nest(super::SUPG002_PATH, protected(super::supg002_routes(), &conn))
        .nest(super::SUPG003_PATH, protected(super::supg003_routes(), &conn))
        .nest(super::SUPG004_PATH, protected(super::supg004_routes(), &conn))
        .nest(super::SUPG005_PATH, protected(super::supg005_routes(), &conn))
        .nest(super::ALPG000_PATH, protected(super::alpg000_routes(), &conn))
        .nest(super::ALPG001_PATH, protected(super::alpg001_routes(), &conn))
        .nest(super::AUDIT_EVENTS_PATH, protected(super::audit_events_routes(), &conn))
        .nest(super::ACCESS_PATH, protected(super::access_routes(), &conn))
        .nest("/hooks", super::kratos_hook_routes());

    // Root router: provides health & probes at both root and under the prefix
    Router::new()
        .merge(base_routes())
        .nest(&clean_prefix, api_router)
        .nest(&format!("/api/v1{}", super::DBPG000_PATH), protected(super::dbpg000_routes(), &conn))
        .nest(&format!("/api/v1{}", super::DBPG001_PATH), protected(super::dbpg001_routes(), &conn))
        .nest(&format!("/api/v1{}", super::DBPG002_PATH), protected(super::dbpg002_routes(), &conn))
        .nest(&format!("/api/v1{}", super::DBPG003_PATH), protected(super::dbpg003_routes(), &conn))
        .nest(&format!("/api/v1{}", super::DBPG004_PATH), protected(super::dbpg004_routes(), &conn))
        .nest(&format!("/api/v1{}", super::DBPG005_PATH), protected(super::dbpg005_routes(), &conn))
        .nest(&format!("/api/v1{}", super::DBPG006_PATH), protected(super::dbpg006_routes(), &conn))
        .nest(&format!("/api/v1{}", super::SUPG001_PATH), protected(super::supg001_routes(), &conn))
        .nest(&format!("/api/v1{}", super::SUPG002_PATH), protected(super::supg002_routes(), &conn))
        .nest(&format!("/api/v1{}", super::SUPG003_PATH), protected(super::supg003_routes(), &conn))
        .nest(&format!("/api/v1{}", super::SUPG004_PATH), protected(super::supg004_routes(), &conn))
        .nest(&format!("/api/v1{}", super::SUPG005_PATH), protected(super::supg005_routes(), &conn))
        .nest(&format!("/api/v1{}", super::ALPG000_PATH), protected(super::alpg000_routes(), &conn))
        .nest(&format!("/api/v1{}", super::ALPG001_PATH), protected(super::alpg001_routes(), &conn))
        .nest(&format!("/api/v1{}", super::AUDIT_EVENTS_PATH), protected(super::audit_events_routes(), &conn))
        .nest(&format!("/api/v1{}", super::ACCESS_PATH), protected(super::access_routes(), &conn))
        .nest("/api/v1/hooks", super::kratos_hook_routes())
        .nest("/api/v1/dashboard", protected(super::dashboard_routes(), &conn))
        .nest("/api/v1/localization", super::localization_routes())
        .nest("/api/v1/i18n", super::localization_routes())
        .layer(axum::middleware::from_fn(infrastructure::wire::cbor_error_responses))
        // Names in query results follow the caller's language
        .layer(axum::middleware::from_fn(set_request_language))
        // After a successful write, clear / refill the Valkey read caches built from that data
        .layer(axum::middleware::from_fn_with_state(
            conn.cache_invalidation.clone(),
            infrastructure::middleware::invalidate_cache_on_write,
        ))
        // Outermost of the error handling: sees the final (possibly CBOR-encoded) error bodies
        .layer(axum::middleware::from_fn_with_state(
            Arc::new(ErrorLocalizer::new(conn.error_templates.clone())),
            localize_error_responses,
        ))
        // Audit trail: just OUTSIDE the error localization so the stored response is the final one
        // the client receives (CBOR-encoded, localized), and inside CORS so preflights never reach
        // it. It sees every protected() rejection (401/503) since those happen below this layer.
        .layer(axum::middleware::from_fn_with_state(
            infrastructure::middleware::AuditTrailState::new(conn.audit_log.clone(), conn.kratos.clone()),
            audit_trail,
        ))
        .layer(cors_layer_from_env())
        .layer(TraceLayer::new_for_http())
        .with_state(conn)
}
