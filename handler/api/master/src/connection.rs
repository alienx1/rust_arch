//! External service connections for api-master (PostgreSQL/pgEdge, Valkey, Ory Kratos, Redpanda,
//! Quickwit, S3) and the registry of feature services built on them (`wiring/`).

use application_core::traits::{CacheService, ContentService, ReadCaches, SqlContext, cache_ns};
use application_master::features::localization::{
    CachedLocalizationQueries, DbErrorTemplateSource, DbLabelQueries, ErrorTemplateSource, I18nCacheRefresher,
    LocalizationQueries,
};
use application_master::features::su::account_security::AccountSecurityCommands;
use infrastructure::middleware::{CacheInvalidation, KratosConfig};
use infrastructure::services::{PgEdgeSqlContext, RedpandaEventService, ValkeyCacheService};
use sea_orm::{ConnectOptions, DatabaseConnection, DbErr};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

use crate::services::Services;
use crate::wiring::{WiringContext, register_all};

/// Unified connection state for `api-master`.
///
/// The infrastructure the middlewares use is a field; every feature's command interface is in
/// [`services`](Self::services), looked up by type, so a route handler just asks for
/// `State<Arc<dyn FeatureCommands>>` (see the `FromRef` impl below and `wiring/`).
#[derive(Clone)]
pub struct AppConnection {
    /// pgEdge / PostgreSQL Database Context
    pub db: PgEdgeSqlContext,
    /// Valkey / Redis Cache Service (optional)
    pub cache: Option<ValkeyCacheService>,
    /// Clears / refills the Valkey read cache after writes (`None` when the cache is off)
    pub cache_invalidation: Option<CacheInvalidation>,
    /// Ory Kratos configuration for auth verification (optional)
    pub kratos: Option<KratosConfig>,
    /// Error message templates (`error.<CODE>` labels) for the error-localization middleware
    pub error_templates: Arc<dyn ErrorTemplateSource>,
    /// Audit log (Quickwit): pages / clicks / API calls of users; a no-op until configured
    pub audit_log: Arc<dyn application_core::traits::AuditLogPort>,
    /// Tells a user how their write went (Redpanda -> api-notification); `None` without a broker client
    pub notifier: Option<Arc<dyn application_core::traits::NotificationService>>,
    /// Every feature's command interface (and the permission checks), by type
    pub services: Services,
}

impl AppConnection {
    /// Initialize all connections from environment variables:
    /// - `DATABASE_URL`: PostgreSQL / pgEdge connection string
    /// - `DATABASE_MAX_CONNECTIONS`: Pool size (default: 20)
    /// - `VALKEY_URL`: Valkey / Redis URL (default: redis://localhost:6379)
    /// - `KRATOS_PUBLIC_URL`: Ory Kratos public URL (default: http://localhost:4433)
    pub async fn init_from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/arch?currentSchema=arch".to_string()
        });
        let max_conns: u32 = env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20);

        info!("Connecting to PostgreSQL/pgEdge at: {database_url}");
        let mut opt = ConnectOptions::new(&database_url);
        opt.max_connections(max_conns)
            .min_connections(2)
            .connect_timeout(Duration::from_secs(10))
            .acquire_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(300))
            .max_lifetime(Duration::from_secs(1800))
            .sqlx_logging(false);
        // Writes carry the API caller into Postgres so the audit trigger can name them
        infrastructure::track_actor(&mut opt);

        let db = PgEdgeSqlContext::with_options(opt).await?;
        info!("Connected to PostgreSQL/pgEdge successfully.");

        // Valkey Cache (Optional)
        let valkey_url =
            env::var("VALKEY_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
        let cache = match ValkeyCacheService::new(&valkey_url).await {
            Ok(c) => {
                info!("Connected to Valkey at: {valkey_url}");
                Some(c)
            }
            Err(e) => {
                warn!("Could not connect to Valkey ({e}), caching disabled.");
                None
            }
        };

        // Read-through cache of the rarely changing reads (web -> API -> Valkey -> database). On
        // unless Valkey is unreachable or READ_CACHE_ENABLED=false; the API works without it.
        let read_cache_on = env::var("READ_CACHE_ENABLED").map(|v| !v.eq_ignore_ascii_case("false")).unwrap_or(true);
        let caches = match (&cache, read_cache_on) {
            (Some(c), true) => ReadCaches::new(Some(Arc::new(c.clone()) as Arc<dyn CacheService>)),
            _ => ReadCaches::disabled(),
        };
        info!("Read cache (Valkey): {}", if caches.is_enabled() { "on" } else { "off" });
        // A summary line per namespace while the cache is in use (CACHE_STATS_SECS, 0 = never)
        let stats_every = env::var("CACHE_STATS_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
        application_core::traits::spawn_stats_logger(caches.clone(), std::time::Duration::from_secs(stats_every));

        // Ory Kratos (Optional)
        let kratos_public_url =
            env::var("KRATOS_PUBLIC_URL").unwrap_or_else(|_| "http://localhost:4433".to_string());
        let kratos_admin_url =
            env::var("KRATOS_ADMIN_URL").unwrap_or_else(|_| "http://localhost:4434".to_string());
        let kratos = {
            let mut cfg = KratosConfig::new(&kratos_public_url).with_admin_url(&kratos_admin_url);
            if let Some(ref c) = cache {
                cfg = cfg.with_cache(c.clone());
            }
            Some(cfg)
        };

        // Dictionaries (and the API error templates built from them) are read through Valkey
        let loc_queries: Arc<dyn LocalizationQueries> = Arc::new(CachedLocalizationQueries::new(
            Arc::new(DbLabelQueries::new(db.connection().clone())),
            caches.ns(cache_ns::I18N),
        ));
        let error_templates: Arc<dyn ErrorTemplateSource> = Arc::new(DbErrorTemplateSource::new(loc_queries.clone()));
        let i18n_refresher = Arc::new(I18nCacheRefresher::new(loc_queries.clone()));

        // Logos and photos live in Garage / S3 (`AWS_*`, `S3_BUCKET_NAME`); the client only connects on use
        let image_content: Arc<dyn ContentService> =
            Arc::new(infrastructure::S3ContentService::new(infrastructure::s3_client_from_env().await));
        let image_bucket = env::var("S3_BUCKET_NAME").unwrap_or_else(|_| "rust-arch-bucket".to_string());

        // Audit log: Quickwit unless AUDIT_LOG_ENABLED=false; constructing never fails (Quickwit may be down)
        let audit_config = infrastructure::QuickwitConfig::from_env();
        let audit_log: Arc<dyn application_core::traits::AuditLogPort> = if audit_config.enabled {
            infrastructure::QuickwitAuditLog::start(audit_config)
        } else {
            Arc::new(application_core::traits::NoopAuditLog)
        };
        // Row changes captured by the database trigger go to the same audit log
        infrastructure::spawn_data_change_shipper(db.connection().clone(), audit_log.clone());

        // Redpanda: report requests, mail and the notices that tell users how their writes went.
        // Creating the client does not connect, so a broker that is down only fails the request.
        let broker = RedpandaEventService::new(&env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string()))
            .map_err(|e| warn!("Redpanda client not available: {e}"))
            .ok()
            .map(Arc::new);
        let notifier: Option<Arc<dyn application_core::traits::NotificationService>> = broker.clone().map(|events| {
            Arc::new(infrastructure::services::RedpandaNotificationPublisher::new(events))
                as Arc<dyn application_core::traits::NotificationService>
        });

        // The features (wiring/<feature>.rs), built from the infrastructure above
        let services = register_all(&WiringContext {
            db: db.connection().clone(),
            cache: cache.clone(),
            caches: caches.clone(),
            kratos: kratos.clone(),
            audit_log: audit_log.clone(),
            broker,
            image_content,
            image_bucket,
            loc_queries,
        });

        // Writes clear (and where possible refill) the cached reads built from the data they edit
        let cache_invalidation = caches.service().map(|service| {
            CacheInvalidation::new(service, crate::cache_policy::invalidation_rules()).with_refresher(i18n_refresher)
        });

        Ok(Self { db, cache, cache_invalidation, kratos, error_templates, audit_log, notifier, services })
    }

    /// A registered service; asking for one that was never wired is a start-up mistake (panics
    /// with the type's name)
    pub fn service<T: ?Sized + Send + Sync + 'static>(&self) -> Arc<T> {
        self.services.require::<T>()
    }

    /// Access the underlying Sea-ORM `DatabaseConnection`.
    #[allow(dead_code)]
    pub fn db_conn(&self) -> &DatabaseConnection {
        self.db.connection()
    }

    /// Health check for database connection.
    pub async fn ping_db(&self) -> Result<(), DbErr> {
        self.db.ping().await
    }

    /// Health check for Valkey cache connection.
    pub async fn ping_cache(&self) -> Result<(), String> {
        if let Some(ref cache) = self.cache {
            cache.ping().await.map_err(|e| e.to_string())
        } else {
            Ok(())
        }
    }
}

/// Every route handler's `State<Arc<dyn FeatureCommands>>` is resolved here, by type, from the
/// registry the `wiring/` files filled: one impl for all features (no per-feature `FromRef`).
impl<T: ?Sized + Send + Sync + 'static> axum::extract::FromRef<AppConnection> for Arc<T> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.services.require::<T>()
    }
}

/// The hooks take the account security commands as optional (a deployment without them still
/// guards logins by the account flag)
impl axum::extract::FromRef<AppConnection> for Option<Arc<dyn AccountSecurityCommands>> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.services.get::<dyn AccountSecurityCommands>()
    }
}
