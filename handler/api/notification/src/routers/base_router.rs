//! Base router for api-notification: probes, the NTPG000 routes behind a Kratos session, CORS, tracing.

use crate::connection::NotificationConnection;
use axum::{Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use infrastructure::middleware::{cors_layer_from_env, require_kratos_session};
use tower_http::trace::TraceLayer;

async fn livez() -> impl IntoResponse {
    (StatusCode::OK, "alive")
}

async fn readyz(State(conn): State<NotificationConnection>) -> impl IntoResponse {
    match conn.ping_db().await {
        Ok(()) => (StatusCode::OK, "ready"),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "database not ready"),
    }
}

fn base_routes() -> Router<NotificationConnection> {
    Router::new().route("/health", get(livez)).route("/livez", get(livez)).route("/readyz", get(readyz))
}

fn protected(routes: Router<NotificationConnection>, conn: &NotificationConnection) -> Router<NotificationConnection> {
    match &conn.kratos {
        Some(kratos) => routes
            .route_layer(axum::middleware::from_fn_with_state(kratos.clone(), require_kratos_session)),
        None => routes.route_layer(axum::middleware::from_fn(
            |_req: axum::extract::Request, _next: axum::middleware::Next| async {
                (StatusCode::SERVICE_UNAVAILABLE, "authentication provider not configured")
            },
        )),
    }
}

/// `prefix` is e.g. `/api/v1/notifications`; the NTPG000 routes sit below it
pub fn build_app_router(conn: NotificationConnection, prefix: &str) -> Router {
    let prefix = if prefix.starts_with('/') { prefix.to_string() } else { format!("/{prefix}") };
    Router::new()
        .merge(base_routes())
        .nest(&prefix, Router::new().merge(base_routes()).merge(protected(super::ntpg000_routes(), &conn)))
        .layer(axum::middleware::from_fn(infrastructure::wire::cbor_error_responses))
        .layer(cors_layer_from_env())
        .layer(TraceLayer::new_for_http())
        .with_state(conn)
}
