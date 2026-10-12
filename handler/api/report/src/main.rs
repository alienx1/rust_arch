//! Report API Service (api-report): builds reports in the background from Redpanda requests and
//! serves them to the web app.
use api_report::{ReportConnection, build_app_router};
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
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,api_report=debug,application_report=debug".into()),
        )
        .init();
    info!("Starting api-report (report worker + API)...");

    let host = env::var("REPORT_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("REPORT_PORT")
        .or_else(|_| env::var("API_REPORT_PORT"))
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8001);
    let prefix = env::var("REPORT_PREFIX")
        .or_else(|_| env::var("API_REPORT_PREFIX"))
        .unwrap_or_else(|_| "/api/v1/reports".to_string());

    let connection = ReportConnection::init_from_env().await?;
    let app = build_app_router(connection, &prefix);

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    info!("Server listening on http://{addr}{prefix}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    info!("api-report shut down cleanly.");
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
