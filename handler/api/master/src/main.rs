//! Master API Service (api-master)
//!
//! Entry point for the Master Data Engine. Manages connections, base routing,
//! and graceful lifecycle shutdown.
use api_master::{AppConnection, build_app_router};
use std::env;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Load .env file
    dotenvy::dotenv().ok();

    // 2. Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,api_master=debug,tower_http=info,arch::cache=debug".into()),
        )
        .init();

    info!("Starting api-master (Master Data Engine)...");

    // 3. Service configuration from environment
    let host = env::var("MASTER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("MASTER_PORT")
        .or_else(|_| env::var("API_MASTER_PORT"))
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8000);
    let prefix = env::var("MASTER_PREFIX")
        .or_else(|_| env::var("API_MASTER_PREFIX"))
        .unwrap_or_else(|_| "/api/v1/master".to_string());

    // 4. Initialize database, cache, and auth connections
    let connection = AppConnection::init_from_env().await?;

    // 5. Build Axum application router
    let app = build_app_router(connection, &prefix);

    // 6. Bind TCP listener and serve
    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    info!("Server listening on http://{addr}");
    info!("Base API prefix: {prefix}");
    info!("Health probe: http://{addr}/health (and http://{addr}{prefix}/health)");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    // Connect info: the audit trail records the socket peer when there is no X-Forwarded-For
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("api-master shut down cleanly.");
    Ok(())
}

/// Graceful shutdown handler waiting for SIGINT (Ctrl+C) or SIGTERM.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received SIGINT (Ctrl+C), initiating graceful shutdown...");
        },
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown...");
        },
    }
}
