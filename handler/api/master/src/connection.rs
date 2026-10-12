//! External service connections for api-master (PostgreSQL/pgEdge, Valkey, Ory Kratos).

use application_core::traits::{CacheService, SqlContext};
use application_master::features::dashboard::{
    DashboardCommandExecutor, DashboardCommands, DbDashboardQueries,
};
use application_master::features::su::supg005::{
    DbSupg005Queries, Supg005CommandExecutor, Supg005Commands,
};
use application_master::features::localization::{
    DbErrorTemplateSource, DbLabelQueries, ErrorTemplateSource, I18nCommandExecutor,
    LocalizationCommands,
};
use application_master::features::db::dbpg000::{
    DbDbpg000Queries, Dbpg000CommandExecutor, Dbpg000Commands, Dbpg000Queries,
};
use application_master::features::su::access_control::{AccessControl, DbAccessControl};
use application_master::features::su::supg001::{
    DbSupg001Queries, Supg001CommandExecutor, Supg001Commands, Supg001Queries,
};
use application_core::traits::{IdentityProvisioningPort, ReadCaches, cache_ns};
use application_master::features::localization::{CachedLocalizationQueries, I18nCacheRefresher};
use infrastructure::middleware::CacheInvalidation;
use application_master::features::al::alpg000::{Alpg000CommandExecutor, Alpg000Commands};
use application_master::features::al::alpg001::{Alpg001CommandExecutor, Alpg001Commands, DbAlpg001Queries};
use application_master::features::su::supg004::{
    DbSupg004Queries, Supg004CommandExecutor, Supg004Commands, Supg004Queries,
};
use application_master::features::su::supg003::{
    DbSupg003Queries, Supg003CommandExecutor, Supg003Commands, Supg003Queries,
};
use application_master::features::su::supg002::{
    DbSupg002Queries, Supg002CommandExecutor, Supg002Commands,
};
use application_master::features::db::dbpg005::{
    DbDbpg005Queries, Dbpg005CommandExecutor, Dbpg005Commands, Dbpg005Queries,
};
use application_master::features::db::dbpg006::{
    DbDbpg006Queries, Dbpg006CommandExecutor, Dbpg006Commands, Dbpg006Queries,
};
use application_master::features::db::dbpg004::{
    DbDbpg004Queries, Dbpg004CommandExecutor, Dbpg004Commands, Dbpg004Queries,
};
use application_master::features::db::dbpg003::{
    DbDbpg003Queries, Dbpg003CommandExecutor, Dbpg003Commands, Dbpg003Queries,
};
use application_master::features::db::dbpg002::{
    DbDbpg002Queries, Dbpg002CommandExecutor, Dbpg002Commands, Dbpg002Queries,
};
use application_master::features::db::dbpg001::{
    DbDbpg001Queries, Dbpg001CommandExecutor, Dbpg001Commands,
};
use infrastructure::middleware::KratosConfig;
use infrastructure::services::{PgEdgeSqlContext, ValkeyCacheService};
use sea_orm::{ConnectOptions, DatabaseConnection, DbErr};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

/// Unified connection state for `api-master`.
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
    /// Localization commands interface
    pub localization_commands: Arc<dyn LocalizationCommands>,
    /// Error message templates (`error.<CODE>` labels) for the error-localization middleware
    pub error_templates: Arc<dyn ErrorTemplateSource>,
    /// Audit log (Quickwit): pages / clicks / API calls of users; a no-op until configured
    pub audit_log: Arc<dyn application_core::traits::AuditLogPort>,
    /// Dashboard commands interface
    pub dashboard_commands: Arc<dyn DashboardCommands>,
    /// IAM commands interface
    pub supg005_commands: Arc<dyn Supg005Commands>,
    /// DBPG000 (organizations & companies list) commands interface
    pub dbpg000_commands: Arc<dyn Dbpg000Commands>,
    /// DBPG001 (organization & company create/update) commands interface
    pub dbpg001_commands: Arc<dyn Dbpg001Commands>,
    /// DBPG002 (branches & departments) commands interface
    pub dbpg002_commands: Arc<dyn Dbpg002Commands>,
    /// DBPG003 (employees) commands interface
    pub dbpg003_commands: Arc<dyn Dbpg003Commands>,
    /// DBPG004 (reference data) commands interface
    pub dbpg004_commands: Arc<dyn Dbpg004Commands>,
    /// DBPG005 (geography) commands interface
    pub dbpg005_commands: Arc<dyn Dbpg005Commands>,
    /// DBPG006 (i18n labels) commands interface
    pub dbpg006_commands: Arc<dyn Dbpg006Commands>,
    /// SUPG001 (systems & modules) commands interface
    pub supg001_commands: Arc<dyn Supg001Commands>,
    /// SUPG002 (menu tree) commands interface
    pub supg002_commands: Arc<dyn Supg002Commands>,
    /// SUPG003 (permission matrix) commands interface
    pub supg003_commands: Arc<dyn Supg003Commands>,
    /// SUPG004 (user accounts) commands interface
    pub supg004_commands: Arc<dyn Supg004Commands>,
    /// ALPG000 (audit log) commands interface
    pub alpg000_commands: Arc<dyn Alpg000Commands>,
    pub alpg001_commands: Arc<dyn Alpg001Commands>,
    /// Permission checks for the security-administration endpoints
    pub access_control: Arc<dyn AccessControl>,
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

        // Localization Commands
        // Dictionaries (and the API error templates built from them) are read through Valkey
        let loc_queries: Arc<dyn application_master::features::localization::LocalizationQueries> =
            Arc::new(CachedLocalizationQueries::new(
                Arc::new(DbLabelQueries::new(db.connection().clone())),
                caches.ns(cache_ns::I18N),
            ));
        let error_templates: Arc<dyn ErrorTemplateSource> =
            Arc::new(DbErrorTemplateSource::new(loc_queries.clone()));
        let i18n_refresher = Arc::new(I18nCacheRefresher::new(loc_queries.clone()));
        let localization_commands: Arc<dyn LocalizationCommands> =
            Arc::new(I18nCommandExecutor::new(loc_queries));

        // Dashboard Commands
        let dash_queries = Arc::new(DbDashboardQueries::new(db.connection().clone()));
        let dashboard_commands: Arc<dyn DashboardCommands> =
            Arc::new(DashboardCommandExecutor::new(dash_queries).with_cache(caches.clone()));

        // SUPG005 Commands
        let supg005_queries = Arc::new(DbSupg005Queries::new(db.connection().clone()));
        let mut supg005_executor = Supg005CommandExecutor::new(supg005_queries);
        if let Some(ref k) = kratos {
            supg005_executor = supg005_executor.with_revocation(Some(Arc::new(k.clone())));
        }
        let supg005_commands: Arc<dyn Supg005Commands> = Arc::new(supg005_executor);

        // DBPG000 / DBPG001 Commands (shared Sea-ORM read runner)
        let dbpg000_queries: Arc<dyn Dbpg000Queries> =
            Arc::new(DbDbpg000Queries::new(db.connection().clone()));
        let dbpg000_commands: Arc<dyn Dbpg000Commands> =
            Arc::new(Dbpg000CommandExecutor::new(dbpg000_queries.clone()).with_cache(caches.clone()));
        let dbpg001_queries = Arc::new(DbDbpg001Queries::new(dbpg000_queries));
        let dbpg001_commands: Arc<dyn Dbpg001Commands> =
            Arc::new(Dbpg001CommandExecutor::new(dbpg001_queries, db.connection().clone()).with_cache(caches.clone()));

        // DBPG002 Commands
        let dbpg002_queries: Arc<dyn Dbpg002Queries> =
            Arc::new(DbDbpg002Queries::new(db.connection().clone()));
        let dbpg002_commands: Arc<dyn Dbpg002Commands> =
            Arc::new(Dbpg002CommandExecutor::new(dbpg002_queries, db.connection().clone()).with_cache(caches.clone()));

        // DBPG003 Commands
        let dbpg003_queries: Arc<dyn Dbpg003Queries> =
            Arc::new(DbDbpg003Queries::new(db.connection().clone()));
        let dbpg003_commands: Arc<dyn Dbpg003Commands> =
            Arc::new(
                Dbpg003CommandExecutor::new(dbpg003_queries, db.connection().clone())
                    .with_cache(caches.clone())
                    .with_account_suspension(supg005_commands.clone()),
            );

        // DBPG004 Commands
        let dbpg004_queries: Arc<dyn Dbpg004Queries> =
            Arc::new(DbDbpg004Queries::new(db.connection().clone()));
        let dbpg004_commands: Arc<dyn Dbpg004Commands> =
            Arc::new(Dbpg004CommandExecutor::new(dbpg004_queries, db.connection().clone()).with_cache(caches.clone()));

        // DBPG005 Commands
        let dbpg005_queries: Arc<dyn Dbpg005Queries> =
            Arc::new(DbDbpg005Queries::new(db.connection().clone()));
        let dbpg005_commands: Arc<dyn Dbpg005Commands> =
            Arc::new(Dbpg005CommandExecutor::new(dbpg005_queries, db.connection().clone()).with_cache(caches.clone()));

        // DBPG006 Commands
        let dbpg006_queries: Arc<dyn Dbpg006Queries> =
            Arc::new(DbDbpg006Queries::new(db.connection().clone()));
        let dbpg006_commands: Arc<dyn Dbpg006Commands> =
            Arc::new(Dbpg006CommandExecutor::new(dbpg006_queries, db.connection().clone()).with_cache(caches.clone()));

        // SUPG001 Commands
        let supg001_queries: Arc<dyn Supg001Queries> =
            Arc::new(DbSupg001Queries::new(db.connection().clone()));
        let supg001_commands: Arc<dyn Supg001Commands> =
            Arc::new(Supg001CommandExecutor::new(supg001_queries, db.connection().clone()));

        // SUPG002 Commands
        let supg002_queries = Arc::new(DbSupg002Queries::new(db.connection().clone()));
        let supg002_commands: Arc<dyn Supg002Commands> =
            Arc::new(Supg002CommandExecutor::new(supg002_queries, db.connection().clone()).with_cache(caches.clone()));

        // SUPG003 Commands
        let supg003_queries: Arc<dyn Supg003Queries> =
            Arc::new(DbSupg003Queries::new(db.connection().clone()));
        let supg003_commands: Arc<dyn Supg003Commands> =
            Arc::new(Supg003CommandExecutor::new(supg003_queries, db.connection().clone()));

        // SUPG004 Commands: accounts are created at the identity provider (Kratos) too
        let supg004_queries: Arc<dyn Supg004Queries> =
            Arc::new(DbSupg004Queries::new(db.connection().clone()));
        let identity: Option<Arc<dyn IdentityProvisioningPort>> = kratos
            .clone()
            .map(|k| Arc::new(k) as Arc<dyn IdentityProvisioningPort>);
        let supg004_commands: Arc<dyn Supg004Commands> = Arc::new(
            Supg004CommandExecutor::new(
                supg004_queries,
                db.connection().clone(),
                identity,
                supg005_commands.clone(),
            )
            .with_cache(caches.clone()),
        );

        // ALPG000 Commands: audit log search + client event intake over the audit store
        // Quickwit unless AUDIT_LOG_ENABLED=false; constructing never fails (Quickwit may be down)
        let audit_config = infrastructure::QuickwitConfig::from_env();
        let audit_log: Arc<dyn application_core::traits::AuditLogPort> = if audit_config.enabled {
            infrastructure::QuickwitAuditLog::start(audit_config)
        } else {
            Arc::new(application_core::traits::NoopAuditLog)
        };
        // Row changes captured by the database trigger go to the same audit log
        infrastructure::spawn_data_change_shipper(db.connection().clone(), audit_log.clone());
        // Report requests (CSV export of the audit log) go to the report worker through Redpanda;
        // creating the producer does not connect, so a broker that is down only fails the request
        let kafka_brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
        let mut alpg000_executor = Alpg000CommandExecutor::new(audit_log.clone()).with_cache(caches.clone()).with_programs(Arc::new(
            application_master::features::al::alpg000::DbAlpg000Queries::new(db.connection().clone()),
        ));
        // The process monitor (ALPG001) reads the same job rows and re-publishes their requests
        let mut alpg001_executor = Alpg001CommandExecutor::new(Arc::new(DbAlpg001Queries::new(db.connection().clone())));
        match infrastructure::services::RedpandaEventService::new(&kafka_brokers) {
            Ok(events) => {
                let events = Arc::new(events);
                alpg000_executor = alpg000_executor.with_events(events.clone());
                alpg001_executor = alpg001_executor.with_events(events);
            }
            Err(e) => warn!("Report queue (Redpanda) not available: {e}"),
        }
        let alpg000_commands: Arc<dyn Alpg000Commands> = Arc::new(alpg000_executor);
        let alpg001_commands: Arc<dyn Alpg001Commands> = Arc::new(alpg001_executor);

        let access_control: Arc<dyn AccessControl> = Arc::new(DbAccessControl::new(db.connection().clone()).with_cache(caches.clone()));

        // Writes clear (and where possible refill) the cached reads built from the data they edit
        let cache_invalidation = caches.service().map(|service| {
            CacheInvalidation::new(service, crate::cache_policy::invalidation_rules()).with_refresher(i18n_refresher)
        });

        Ok(Self {
            db,
            cache,
            cache_invalidation,
            kratos,
            localization_commands,
            error_templates,
            audit_log,
            dashboard_commands,
            supg005_commands,
            dbpg000_commands,
            dbpg001_commands,
            dbpg002_commands,
            dbpg003_commands,
            dbpg004_commands,
            dbpg005_commands,
            dbpg006_commands,
            supg001_commands,
            supg002_commands,
            supg003_commands,
            supg004_commands,
            alpg000_commands,
            alpg001_commands,
            access_control,
        })
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

impl axum::extract::FromRef<AppConnection> for Arc<dyn LocalizationCommands> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.localization_commands.clone()
    }
}

impl axum::extract::FromRef<AppConnection> for Arc<dyn DashboardCommands> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.dashboard_commands.clone()
    }
}

impl axum::extract::FromRef<AppConnection> for Arc<dyn Supg005Commands> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.supg005_commands.clone()
    }
}

impl axum::extract::FromRef<AppConnection> for Arc<dyn Dbpg000Commands> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.dbpg000_commands.clone()
    }
}

impl axum::extract::FromRef<AppConnection> for Arc<dyn Dbpg001Commands> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.dbpg001_commands.clone()
    }
}

impl axum::extract::FromRef<AppConnection> for Arc<dyn Dbpg002Commands> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.dbpg002_commands.clone()
    }
}

impl axum::extract::FromRef<AppConnection> for Arc<dyn Dbpg003Commands> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.dbpg003_commands.clone()
    }
}

impl axum::extract::FromRef<AppConnection> for Arc<dyn Supg002Commands> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.supg002_commands.clone()
    }
}

impl axum::extract::FromRef<AppConnection> for Arc<dyn Dbpg004Commands> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.dbpg004_commands.clone()
    }
}

impl axum::extract::FromRef<AppConnection> for Arc<dyn Dbpg005Commands> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.dbpg005_commands.clone()
    }
}

impl axum::extract::FromRef<AppConnection> for Arc<dyn Dbpg006Commands> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.dbpg006_commands.clone()
    }
}

impl axum::extract::FromRef<AppConnection> for Arc<dyn AccessControl> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.access_control.clone()
    }
}

impl axum::extract::FromRef<AppConnection> for Arc<dyn Supg001Commands> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.supg001_commands.clone()
    }
}

impl axum::extract::FromRef<AppConnection> for Arc<dyn Supg003Commands> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.supg003_commands.clone()
    }
}

impl axum::extract::FromRef<AppConnection> for Arc<dyn Supg004Commands> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.supg004_commands.clone()
    }
}

impl axum::extract::FromRef<AppConnection> for Arc<dyn Alpg001Commands> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.alpg001_commands.clone()
    }
}

impl axum::extract::FromRef<AppConnection> for Arc<dyn Alpg000Commands> {
    fn from_ref(conn: &AppConnection) -> Self {
        conn.alpg000_commands.clone()
    }
}
