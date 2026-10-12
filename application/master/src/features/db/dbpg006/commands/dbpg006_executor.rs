//! DBPG006 Command Executor: label writes and reads; each action lives in its own file.

use application_core::traits::{ReadCaches, cache_ns, cache_ttl};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

use crate::features::db::dbpg006::commands::Dbpg006Commands;
use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::{
    CreateLabelCommand, LabelCategoryDto, LabelEntryDto, LabelFilterQuery, LabelPageDto,
    LanguageOptionDto, UpdateLabelCommand,
};
use crate::features::db::dbpg006::queries::Dbpg006Queries;

pub struct Dbpg006CommandExecutor {
    pub(super) queries: Arc<dyn Dbpg006Queries>,
    pub(super) db: DatabaseConnection,
    /// Valkey read-through for the rarely changing reads (off until `with_cache`)
    pub(super) caches: ReadCaches,
}

impl Dbpg006CommandExecutor {
    pub fn new(queries: Arc<dyn Dbpg006Queries>, db: DatabaseConnection) -> Self {
        Self { queries, db, caches: ReadCaches::disabled() }
    }

    /// Reads that change rarely are served from Valkey; writes clear them (see the cache-invalidation middleware)
    pub fn with_cache(mut self, caches: ReadCaches) -> Self {
        self.caches = caches;
        self
    }
}

#[async_trait]
impl Dbpg006Commands for Dbpg006CommandExecutor {
    async fn list_languages(&self) -> Result<Vec<LanguageOptionDto>, Dbpg006Error> {
        let cache = self.caches.ns(cache_ns::I18N);
        let key = cache.key("list_languages", &());
        cache.through_key(key, cache_ttl::I18N, || self.list_languages()).await
    }

    async fn list_categories(&self) -> Result<Vec<LabelCategoryDto>, Dbpg006Error> {
        let cache = self.caches.ns(cache_ns::I18N);
        let key = cache.key("list_categories", &());
        cache.through_key(key, cache_ttl::I18N, || self.list_categories()).await
    }

    async fn list_labels(&self, filter: LabelFilterQuery) -> Result<LabelPageDto, Dbpg006Error> {
        self.list_labels(filter).await
    }

    async fn get_label(
        &self,
        category: &str,
        key: &str,
    ) -> Result<Option<LabelEntryDto>, Dbpg006Error> {
        self.get_label(category, key).await
    }

    async fn create_label(
        &self,
        cmd: CreateLabelCommand,
        actor_id: Option<Uuid>,
    ) -> Result<LabelEntryDto, Dbpg006Error> {
        self.create_label(cmd, actor_id).await
    }

    async fn update_label(
        &self,
        cmd: UpdateLabelCommand,
        actor_id: Option<Uuid>,
    ) -> Result<LabelEntryDto, Dbpg006Error> {
        self.update_label(cmd, actor_id).await
    }

    async fn delete_label(&self, category: &str, key: &str) -> Result<(), Dbpg006Error> {
        self.delete_label(category, key).await
    }
}
