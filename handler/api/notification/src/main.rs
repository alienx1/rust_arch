//! Notification API Service (api-notification): stores the notifications services publish to
//! Redpanda and pushes them to the recipients' open web sessions.
use api_notification::{NotificationConnection, build_app_router};
use std::env;
use std::net::SocketAddr;
use std::time::Duration;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

/// How long open connections get to finish before the process exits anyway
const SHUTDOWN_GRACE: Duration = Duration::from_secs(10);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,api_notification=debug,application_notification=debug".into()),
        )
        .init();
    info!("Starting api-notification (notification service)...");

    let host = env::var("NOTIFICATION_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("NOTIFICATION_PORT")
        .or_else(|_| env::var("API_NOTIFICATION_PORT"))
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8002);
    let prefix = env::var("NOTIFICATION_PREFIX")
        .or_else(|_| env::var("API_NOTIFICATION_PREFIX"))
        .unwrap_or_else(|_| "/api/v1/notifications".to_string());

    let connection = NotificationConnection::init_from_env().await?;
    let sessions = connection.sessions.clone();
    let app = build_app_router(connection, &prefix);

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    info!("Server listening on http://{addr}{prefix}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            // Open push streams never end by themselves and would hold the shutdown for ever
            sessions.close_all();
        })
        .await?;
    info!("api-notification shut down cleanly.");
    Ok(())
}

/// Waits for SIGINT (Ctrl+C) or SIGTERM. After it a watchdog forces the exit if the graceful
/// shutdown has not finished in [`SHUTDOWN_GRACE`], and a second Ctrl+C exits at once.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("Failed to install Ctrl+C signal handler");
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
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
    info!("Shutdown requested, closing connections (Ctrl+C again to force)...");

    tokio::spawn(async {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => warn!("Second Ctrl+C: exiting now"),
            _ = tokio::time::sleep(SHUTDOWN_GRACE) => warn!("Graceful shutdown took longer than {SHUTDOWN_GRACE:?}: exiting"),
        }
        std::process::exit(1);
    });
}
