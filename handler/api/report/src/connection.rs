//! External service connections for api-report (PostgreSQL, Valkey, Kratos, Quickwit, Redpanda,
//! S3) and the report worker that runs on them.

use application_core::traits::{
    AuditLogPort, ContentService, EventConsumer, EventHandler, NoopAuditLog, NotificationService,
    REPORT_REQUESTS_TOPIC, REPORT_WORKER_GROUP,
};
use application_report::features::rppg000::{
    DbRppg000Queries, ReportJobHandler, Rppg000CommandExecutor, Rppg000Commands,
};
use infrastructure::middleware::KratosConfig;
use infrastructure::services::{
    PgEdgeSqlContext, QuickwitAuditLog, QuickwitConfig, RedpandaEventConsumer, RedpandaEventService,
    RedpandaNotificationPublisher, S3ContentService, ValkeyCacheService, s3_client_from_env,
};
use sea_orm::{ConnectOptions, DatabaseConnection};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct ReportConnection {
    pub db: DatabaseConnection,
    pub kratos: Option<KratosConfig>,
    pub rppg000_commands: Arc<dyn Rppg000Commands>,
}

impl ReportConnection {
    /// Connects everything from the environment and starts the report worker:
    /// `DATABASE_URL`, `VALKEY_URL`, `KRATOS_PUBLIC_URL` / `KRATOS_ADMIN_URL`, `QUICKWIT_URL`,
    /// `KAFKA_BROKERS`, `AWS_*` and `S3_BUCKET_NAME`.
    pub async fn init_from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/arch?currentSchema=arch".to_string());
        let mut opt = ConnectOptions::new(&database_url);
        opt.max_connections(10)
            .min_connections(1)
            .connect_timeout(Duration::from_secs(10))
            .acquire_timeout(Duration::from_secs(10))
            .sqlx_logging(false);
        let db = PgEdgeSqlContext::with_options(opt).await?;
        let db = {
            use application_core::traits::SqlContext;
            db.connection().clone()
        };
        info!("Connected to PostgreSQL");

        let valkey_url = env::var("VALKEY_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
        let cache = match ValkeyCacheService::new(&valkey_url).await {
            Ok(c) => Some(c),
            Err(e) => {
                warn!("Could not connect to Valkey ({e}), session cache disabled.");
                None
            }
        };
        let kratos_public = env::var("KRATOS_PUBLIC_URL").unwrap_or_else(|_| "http://localhost:4433".to_string());
        let kratos_admin = env::var("KRATOS_ADMIN_URL").unwrap_or_else(|_| "http://localhost:4434".to_string());
        let mut kratos = KratosConfig::new(&kratos_public).with_admin_url(&kratos_admin);
        if let Some(c) = cache {
            kratos = kratos.with_cache(c);
        }

        // Where the events are read from
        let quickwit = QuickwitConfig::from_env();
        let audit: Arc<dyn AuditLogPort> =
            if quickwit.enabled { QuickwitAuditLog::start(quickwit) } else { Arc::new(NoopAuditLog) };

        let bucket = env::var("S3_BUCKET_NAME").unwrap_or_else(|_| "rust-arch-bucket".to_string());
        let content: Arc<dyn ContentService> = Arc::new(S3ContentService::new(s3_client_from_env().await));
        // Users are told through the notification service: the notification goes to Redpanda,
        // creating the producer does not connect, so a broker that is down only costs the notice
        let kafka_brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
        let notifier: Arc<dyn NotificationService> =
            Arc::new(RedpandaNotificationPublisher::new(Arc::new(RedpandaEventService::new(&kafka_brokers)?)));

        let commands: Arc<dyn Rppg000Commands> = Arc::new(Rppg000CommandExecutor::new(
            Arc::new(DbRppg000Queries::new(db.clone())),
            audit,
            content,
            notifier,
            bucket,
        ));

        spawn_report_worker(commands.clone());
        Ok(Self { db, kratos: Some(kratos), rppg000_commands: commands })
    }

    pub async fn ping_db(&self) -> Result<(), sea_orm::DbErr> {
        self.db.ping().await
    }
}

/// Reads `report.requests` for as long as the process lives; a dead connection is retried
fn spawn_report_worker(commands: Arc<dyn Rppg000Commands>) {
    let brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    tokio::spawn(async move {
        let consumer = RedpandaEventConsumer::new(&brokers);
        let handler: Arc<dyn EventHandler> = Arc::new(ReportJobHandler::new(commands));
        loop {
            if let Err(e) = consumer.consume(REPORT_REQUESTS_TOPIC, REPORT_WORKER_GROUP, handler.clone()).await {
                error!(error = %e, "report worker stopped, retrying in 5 s");
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
    info!(topic = REPORT_REQUESTS_TOPIC, "report worker started");
}
