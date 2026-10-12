use crate::features::dashboard::commands::DashboardCommands;
use crate::features::dashboard::errors::DashboardError;
use crate::features::dashboard::models::DashboardSummary;
use crate::features::dashboard::queries::DashboardQueries;
use application_core::traits::{ReadCaches, cache_ns, cache_ttl};
use async_trait::async_trait;
use std::sync::Arc;

/// Executor implementing the `DashboardCommands` trait contract.
pub struct DashboardCommandExecutor {
    queries: Arc<dyn DashboardQueries>,
    /// Valkey read-through for the rarely changing reads (off until `with_cache`)
    pub(super) caches: ReadCaches,
}

impl DashboardCommandExecutor {
    pub fn new(queries: Arc<dyn DashboardQueries>) -> Self {
        Self { queries, caches: ReadCaches::disabled() }
    }

    /// Reads that change rarely are served from Valkey; writes clear them (see the cache-invalidation middleware)
    pub fn with_cache(mut self, caches: ReadCaches) -> Self {
        self.caches = caches;
        self
    }
}

#[async_trait]
impl DashboardCommands for DashboardCommandExecutor {
    async fn get_dashboard_summary(&self) -> Result<DashboardSummary, DashboardError> {
        let cache = self.caches.ns(cache_ns::DASHBOARD);
        let key = cache.key("summary", &());
        cache.through_key(key, cache_ttl::DASHBOARD, || self.queries.fetch_summary()).await
    }
}
