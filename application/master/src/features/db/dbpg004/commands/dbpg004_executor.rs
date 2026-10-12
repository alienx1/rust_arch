//! DBPG004 Command Executor: lookup writes and reads; each action lives in its own file.

use application_core::traits::{ReadCaches, cache_ns, cache_ttl};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

use crate::features::db::dbpg004::commands::Dbpg004Commands;
use crate::features::db::dbpg004::errors::Dbpg004Error;
use crate::features::db::dbpg004::models::{
    CreateLookupCommand, Dbpg004SummaryDto, LookupCategoryDto, LookupDto, LookupFilterQuery,
    LookupPageDto, UpdateLookupCommand,
};
use crate::features::db::dbpg004::queries::Dbpg004Queries;

pub struct Dbpg004CommandExecutor {
    pub(super) queries: Arc<dyn Dbpg004Queries>,
    pub(super) db: DatabaseConnection,
    /// Valkey read-through for the rarely changing reads (off until `with_cache`)
    pub(super) caches: ReadCaches,
}

impl Dbpg004CommandExecutor {
    pub fn new(queries: Arc<dyn Dbpg004Queries>, db: DatabaseConnection) -> Self {
        Self { queries, db, caches: ReadCaches::disabled() }
    }

    /// Reads that change rarely are served from Valkey; writes clear them (see the cache-invalidation middleware)
    pub fn with_cache(mut self, caches: ReadCaches) -> Self {
        self.caches = caches;
        self
    }
}

#[async_trait]
impl Dbpg004Commands for Dbpg004CommandExecutor {
    async fn list_categories(&self) -> Result<Vec<LookupCategoryDto>, Dbpg004Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("list_categories", &());
        cache.through_key(key, cache_ttl::REF, || self.list_categories()).await
    }

    async fn list_lookups(&self, filter: LookupFilterQuery) -> Result<LookupPageDto, Dbpg004Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("list_lookups", &filter);
        cache.through_key(key, cache_ttl::REF, || self.list_lookups(filter)).await
    }

    async fn get_lookup_by_id(&self, id: Uuid) -> Result<Option<LookupDto>, Dbpg004Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("get_lookup_by_id", &id);
        cache.through_key(key, cache_ttl::REF, || self.get_lookup_by_id(id)).await
    }

    async fn get_summary(&self) -> Result<Dbpg004SummaryDto, Dbpg004Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("get_summary", &());
        cache.through_key(key, cache_ttl::REF, || self.get_summary()).await
    }

    async fn create_lookup(
        &self,
        cmd: CreateLookupCommand,
        actor_id: Option<Uuid>,
    ) -> Result<LookupDto, Dbpg004Error> {
        self.create_lookup(cmd, actor_id).await
    }

    async fn update_lookup(
        &self,
        id: Uuid,
        cmd: UpdateLookupCommand,
        actor_id: Option<Uuid>,
    ) -> Result<LookupDto, Dbpg004Error> {
        self.update_lookup(id, cmd, actor_id).await
    }

    async fn delete_lookup(&self, id: Uuid) -> Result<(), Dbpg004Error> {
        self.delete_lookup(id).await
    }
}
