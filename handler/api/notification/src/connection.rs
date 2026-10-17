//! External service connections for api-notification (PostgreSQL, Valkey, Kratos, Redpanda) and
//! the consumer that stores and pushes notifications.

use application_core::traits::{
    EventConsumer, EventHandler, MAIL_SERVICE_GROUP, MAIL_TOPIC, MailService, NOTIFICATION_SERVICE_GROUP,
    NOTIFICATIONS_TOPIC, NotificationPush,
};
use application_notification::features::mail_delivery::MailDeliveryHandler;
use application_notification::features::ntpg000::{
    DbNtpg000Queries, NotificationMessageHandler, Ntpg000CommandExecutor, Ntpg000Commands,
};
use infrastructure::middleware::KratosConfig;
use infrastructure::services::{
    PgEdgeSqlContext, RedpandaEventConsumer, SseNotificationHub, ValkeyCacheService,
};
use sea_orm::{ConnectOptions, DatabaseConnection};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct NotificationConnection {
    pub db: DatabaseConnection,
    pub kratos: Option<KratosConfig>,
    pub ntpg000_commands: Arc<dyn Ntpg000Commands>,
    /// The open sessions (the SSE endpoint subscribes here)
    pub sessions: Arc<SseNotificationHub>,
}

impl NotificationConnection {
    /// Connects everything from the environment and starts the consumer:
    /// `DATABASE_URL`, `VALKEY_URL`, `KRATOS_PUBLIC_URL` / `KRATOS_ADMIN_URL`, `KAFKA_BROKERS` and
    /// `NOTIFICATION_CONSUMER_GROUP` (default `notification-service`; with several instances give
    /// each its own, every instance has to see every message).
    pub async fn init_from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/arch?currentSchema=arch".to_string());
        let mut opt = ConnectOptions::new(&database_url);
        opt.max_connections(10)
            .min_connections(1)
            .connect_timeout(Duration::from_secs(10))
            .acquire_timeout(Duration::from_secs(10))
            .sqlx_logging(false);
        let db = {
            use application_core::traits::SqlContext;
            PgEdgeSqlContext::with_options(opt).await?.connection().clone()
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

        let sessions = Arc::new(SseNotificationHub::new());
        let push: Arc<dyn NotificationPush> = sessions.clone();
        let commands: Arc<dyn Ntpg000Commands> =
            Arc::new(Ntpg000CommandExecutor::new(Arc::new(DbNtpg000Queries::new(db.clone())), push));

        spawn_consumer(commands.clone());
        // Mail queued by the other services is sent here, with the SMTP settings of su_configuration
        let mail: Arc<dyn MailService> = Arc::new(infrastructure::ConfiguredMailService::new(db.clone()));
        spawn_mail_consumer(Arc::new(MailDeliveryHandler::new(mail).with_failure_notices(commands.clone())));
        Ok(Self { db, kratos: Some(kratos), ntpg000_commands: commands, sessions })
    }

    pub async fn ping_db(&self) -> Result<(), sea_orm::DbErr> {
        self.db.ping().await
    }
}

/// Reads `notifications` for as long as the process lives; a dead connection is retried
fn spawn_consumer(commands: Arc<dyn Ntpg000Commands>) {
    let brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let group = env::var("NOTIFICATION_CONSUMER_GROUP").unwrap_or_else(|_| NOTIFICATION_SERVICE_GROUP.to_string());
    tokio::spawn(async move {
        let consumer = RedpandaEventConsumer::new(&brokers);
        let handler: Arc<dyn EventHandler> = Arc::new(NotificationMessageHandler::new(commands));
        loop {
            if let Err(e) = consumer.consume(NOTIFICATIONS_TOPIC, &group, handler.clone()).await {
                error!(error = %e, "notification consumer stopped, retrying in 5 s");
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
    info!(topic = NOTIFICATIONS_TOPIC, "notification consumer started");
}

/// Reads `mail-outbox` for as long as the process lives and sends what it finds
fn spawn_mail_consumer(handler: Arc<MailDeliveryHandler>) {
    let brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let group = env::var("MAIL_CONSUMER_GROUP").unwrap_or_else(|_| MAIL_SERVICE_GROUP.to_string());
    tokio::spawn(async move {
        let consumer = RedpandaEventConsumer::new(&brokers);
        let handler: Arc<dyn EventHandler> = handler;
        loop {
            if let Err(e) = consumer.consume(MAIL_TOPIC, &group, handler.clone()).await {
                error!(error = %e, "mail consumer stopped, retrying in 5 s");
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
    info!(topic = MAIL_TOPIC, "mail delivery started");
}
