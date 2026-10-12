//! DBPG001 Command Executor: create/update/delete organizations and companies

use application_core::traits::{ReadCaches, cache_ns, cache_ttl};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

use crate::features::db::dbpg001::commands::Dbpg001Commands;
use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::{
    CompanyDto, CompanyTypeLookupDto, CreateCompanyCommand, CreateOrganizationCommand,
    OrganizationDto, UpdateCompanyCommand, UpdateOrganizationCommand,
};
use crate::features::db::dbpg001::queries::Dbpg001Queries;

/// Executor persisting organization and company writes; lookups delegate to `Dbpg001Queries`.
pub struct Dbpg001CommandExecutor {
    pub(super) queries: Arc<dyn Dbpg001Queries>,
    pub(super) db: DatabaseConnection,
    /// Valkey read-through for the rarely changing reads (off until `with_cache`)
    pub(super) caches: ReadCaches,
}

impl Dbpg001CommandExecutor {
    pub fn new(queries: Arc<dyn Dbpg001Queries>, db: DatabaseConnection) -> Self {
        Self { queries, db, caches: ReadCaches::disabled() }
    }

    /// Reads that change rarely are served from Valkey; writes clear them (see the cache-invalidation middleware)
    pub fn with_cache(mut self, caches: ReadCaches) -> Self {
        self.caches = caches;
        self
    }
}

#[async_trait]
impl Dbpg001Commands for Dbpg001CommandExecutor {
    async fn list_organizations(&self) -> Result<Vec<OrganizationDto>, Dbpg001Error> {
        self.list_organizations().await
    }

    async fn get_organization_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<OrganizationDto>, Dbpg001Error> {
        self.get_organization_by_id(id).await
    }

    async fn get_company_by_id(&self, id: Uuid) -> Result<Option<CompanyDto>, Dbpg001Error> {
        self.get_company_by_id(id).await
    }

    async fn list_company_types(&self) -> Result<Vec<CompanyTypeLookupDto>, Dbpg001Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("list_company_types", &());
        cache.through_key(key, cache_ttl::REF, || self.list_company_types()).await
    }

    async fn create_organization(
        &self,
        cmd: CreateOrganizationCommand,
        actor_id: Option<Uuid>,
    ) -> Result<OrganizationDto, Dbpg001Error> {
        self.create_organization(cmd, actor_id).await
    }

    async fn update_organization(
        &self,
        id: Uuid,
        cmd: UpdateOrganizationCommand,
        actor_id: Option<Uuid>,
    ) -> Result<OrganizationDto, Dbpg001Error> {
        self.update_organization(id, cmd, actor_id).await
    }

    async fn create_company(
        &self,
        cmd: CreateCompanyCommand,
        actor_id: Option<Uuid>,
    ) -> Result<CompanyDto, Dbpg001Error> {
        self.create_company(cmd, actor_id).await
    }

    async fn update_company(
        &self,
        id: Uuid,
        cmd: UpdateCompanyCommand,
        actor_id: Option<Uuid>,
    ) -> Result<CompanyDto, Dbpg001Error> {
        self.update_company(id, cmd, actor_id).await
    }

    async fn delete_organization(&self, id: Uuid) -> Result<(), Dbpg001Error> {
        self.delete_organization(id).await
    }

    async fn delete_company(&self, id: Uuid) -> Result<(), Dbpg001Error> {
        self.delete_company(id).await
    }
}
