//! DBPG005 Command Executor: validates and persists geography writes (a unit and its labels in
//! one transaction). Reads delegate to `Dbpg005Queries`. Each action lives in its own file.

use application_core::traits::{ReadCaches, cache_ns, cache_ttl};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

use crate::features::db::dbpg005::commands::Dbpg005Commands;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{
    CreateDistrictCommand, CreatePostalCodeCommand, CreateProvinceCommand,
    CreateSubDistrictCommand, Dbpg005SummaryDto, GeoFilterQuery, GeoNodeDto, GeoPageDto,
    PostalCodeDto, PostalFilterQuery, PostalPageDto, UpdateGeoNodeCommand, UpdatePostalCodeCommand,
};
use crate::features::db::dbpg005::queries::Dbpg005Queries;

pub struct Dbpg005CommandExecutor {
    pub(super) queries: Arc<dyn Dbpg005Queries>,
    pub(super) db: DatabaseConnection,
    /// Valkey read-through for the rarely changing reads (off until `with_cache`)
    pub(super) caches: ReadCaches,
}

impl Dbpg005CommandExecutor {
    pub fn new(queries: Arc<dyn Dbpg005Queries>, db: DatabaseConnection) -> Self {
        Self { queries, db, caches: ReadCaches::disabled() }
    }

    /// Reads that change rarely are served from Valkey; writes clear them (see the cache-invalidation middleware)
    pub fn with_cache(mut self, caches: ReadCaches) -> Self {
        self.caches = caches;
        self
    }
}

#[async_trait]
impl Dbpg005Commands for Dbpg005CommandExecutor {
    async fn list_provinces(&self, filter: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("list_provinces", &filter);
        cache.through_key(key, cache_ttl::REF, || self.list_provinces(filter)).await
    }

    async fn list_districts(&self, filter: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("list_districts", &filter);
        cache.through_key(key, cache_ttl::REF, || self.list_districts(filter)).await
    }

    async fn list_sub_districts(&self, filter: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("list_sub_districts", &filter);
        cache.through_key(key, cache_ttl::REF, || self.list_sub_districts(filter)).await
    }

    async fn list_postal_codes(&self, filter: PostalFilterQuery) -> Result<PostalPageDto, Dbpg005Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("list_postal_codes", &filter);
        cache.through_key(key, cache_ttl::REF, || self.list_postal_codes(filter)).await
    }

    async fn get_province(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("get_province", &id);
        cache.through_key(key, cache_ttl::REF, || self.get_province(id)).await
    }

    async fn get_district(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("get_district", &id);
        cache.through_key(key, cache_ttl::REF, || self.get_district(id)).await
    }

    async fn get_sub_district(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("get_sub_district", &id);
        cache.through_key(key, cache_ttl::REF, || self.get_sub_district(id)).await
    }

    async fn get_postal_code(&self, id: Uuid) -> Result<Option<PostalCodeDto>, Dbpg005Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("get_postal_code", &id);
        cache.through_key(key, cache_ttl::REF, || self.get_postal_code(id)).await
    }

    async fn get_summary(&self) -> Result<Dbpg005SummaryDto, Dbpg005Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("get_summary", &());
        cache.through_key(key, cache_ttl::REF, || self.get_summary()).await
    }

    async fn create_province(
        &self,
        cmd: CreateProvinceCommand,
        actor_id: Option<Uuid>,
    ) -> Result<GeoNodeDto, Dbpg005Error> {
        self.create_province(cmd, actor_id).await
    }

    async fn create_district(
        &self,
        cmd: CreateDistrictCommand,
        actor_id: Option<Uuid>,
    ) -> Result<GeoNodeDto, Dbpg005Error> {
        self.create_district(cmd, actor_id).await
    }

    async fn create_sub_district(
        &self,
        cmd: CreateSubDistrictCommand,
        actor_id: Option<Uuid>,
    ) -> Result<GeoNodeDto, Dbpg005Error> {
        self.create_sub_district(cmd, actor_id).await
    }

    async fn update_province(
        &self,
        id: Uuid,
        cmd: UpdateGeoNodeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<GeoNodeDto, Dbpg005Error> {
        self.update_province(id, cmd, actor_id).await
    }

    async fn update_district(
        &self,
        id: Uuid,
        cmd: UpdateGeoNodeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<GeoNodeDto, Dbpg005Error> {
        self.update_district(id, cmd, actor_id).await
    }

    async fn update_sub_district(
        &self,
        id: Uuid,
        cmd: UpdateGeoNodeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<GeoNodeDto, Dbpg005Error> {
        self.update_sub_district(id, cmd, actor_id).await
    }

    async fn create_postal_code(
        &self,
        cmd: CreatePostalCodeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<PostalCodeDto, Dbpg005Error> {
        self.create_postal_code(cmd, actor_id).await
    }

    async fn update_postal_code(
        &self,
        id: Uuid,
        cmd: UpdatePostalCodeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<PostalCodeDto, Dbpg005Error> {
        self.update_postal_code(id, cmd, actor_id).await
    }

    async fn delete_province(&self, id: Uuid) -> Result<(), Dbpg005Error> {
        self.delete_province(id).await
    }

    async fn delete_district(&self, id: Uuid) -> Result<(), Dbpg005Error> {
        self.delete_district(id).await
    }

    async fn delete_sub_district(&self, id: Uuid) -> Result<(), Dbpg005Error> {
        self.delete_sub_district(id).await
    }

    async fn delete_postal_code(&self, id: Uuid) -> Result<(), Dbpg005Error> {
        self.delete_postal_code(id).await
    }
}
