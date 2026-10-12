//! DBPG000 Command Executor: delegates reads to `Dbpg000Queries`

use application_core::traits::{ReadCaches, cache_ns, cache_ttl};
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::features::db::dbpg000::commands::Dbpg000Commands;
use crate::features::db::dbpg000::errors::Dbpg000Error;
use crate::features::db::dbpg000::models::{
    CompanyDto, CompanyFilterQuery, CompanyTypeLookupDto, OrganizationDto, OrganizationSummaryDto,
};
use crate::features::db::dbpg000::queries::Dbpg000Queries;

pub struct Dbpg000CommandExecutor {
    pub(super) queries: Arc<dyn Dbpg000Queries>,
    /// Valkey read-through for the rarely changing reads (off until `with_cache`)
    pub(super) caches: ReadCaches,
}

impl Dbpg000CommandExecutor {
    pub fn new(queries: Arc<dyn Dbpg000Queries>) -> Self {
        Self { queries, caches: ReadCaches::disabled() }
    }

    /// Reads that change rarely are served from Valkey; writes clear them (see the cache-invalidation middleware)
    pub fn with_cache(mut self, caches: ReadCaches) -> Self {
        self.caches = caches;
        self
    }
}

#[async_trait]
impl Dbpg000Commands for Dbpg000CommandExecutor {
    async fn list_organizations(&self) -> Result<Vec<OrganizationDto>, Dbpg000Error> {
        self.list_organizations().await
    }

    async fn get_organization_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<OrganizationDto>, Dbpg000Error> {
        self.get_organization_by_id(id).await
    }

    async fn list_companies(
        &self,
        filter: CompanyFilterQuery,
    ) -> Result<Vec<CompanyDto>, Dbpg000Error> {
        self.list_companies(filter).await
    }

    async fn get_company_by_id(&self, id: Uuid) -> Result<Option<CompanyDto>, Dbpg000Error> {
        self.get_company_by_id(id).await
    }

    async fn list_company_types(&self) -> Result<Vec<CompanyTypeLookupDto>, Dbpg000Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("list_company_types", &());
        cache.through_key(key, cache_ttl::REF, || self.list_company_types()).await
    }

    async fn get_summary(&self) -> Result<OrganizationSummaryDto, Dbpg000Error> {
        let cache = self.caches.ns(cache_ns::ORG);
        let key = cache.key("get_summary", &());
        cache.through_key(key, cache_ttl::ORG, || self.get_summary()).await
    }
}
