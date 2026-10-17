//! DBPG001 Command Executor: create/update/delete organizations and companies

use application_core::traits::{ContentService, ReadCaches, cache_ns, cache_ttl};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

use crate::features::db::dbpg001::commands::Dbpg001Commands;
use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::{
    CompanyDto, CompanyTypeLookupDto, CreateCompanyCommand, CreateOrganizationCommand, ImageOwner,
    OrganizationDto, StoredImage, UpdateCompanyCommand, UpdateOrganizationCommand,
};
use crate::features::db::dbpg001::queries::Dbpg001Queries;
use crate::features::db::image_object_store::ImageObjectStore;

/// Executor persisting organization and company writes; lookups delegate to `Dbpg001Queries`.
pub struct Dbpg001CommandExecutor {
    pub(super) queries: Arc<dyn Dbpg001Queries>,
    pub(super) db: DatabaseConnection,
    /// Valkey read-through for the rarely changing reads (off until `with_cache`)
    pub(super) caches: ReadCaches,
    /// Object storage and bucket for the logos (images are refused until `with_images`)
    pub(super) images: Option<ImageObjectStore>,
}

impl Dbpg001CommandExecutor {
    pub fn new(queries: Arc<dyn Dbpg001Queries>, db: DatabaseConnection) -> Self {
        Self { queries, db, caches: ReadCaches::disabled(), images: None }
    }

    /// Reads that change rarely are served from Valkey; writes clear them (see the cache-invalidation middleware)
    pub fn with_cache(mut self, caches: ReadCaches) -> Self {
        self.caches = caches;
        self
    }

    /// Logos of organizations and companies are kept in `bucket` of `content`
    pub fn with_images(mut self, content: Arc<dyn ContentService>, bucket: impl Into<String>) -> Self {
        self.images = Some(ImageObjectStore::new(content, bucket));
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

    async fn set_organization_image(&self, id: Uuid, bytes: Vec<u8>) -> Result<OrganizationDto, Dbpg001Error> {
        self.set_image(ImageOwner::Organization, id, bytes).await?;
        self.queries
            .get_organization_by_id(id)
            .await?
            .ok_or_else(|| Dbpg001Error::NotFound(id.to_string()))
    }

    async fn remove_organization_image(&self, id: Uuid) -> Result<OrganizationDto, Dbpg001Error> {
        self.remove_image(ImageOwner::Organization, id).await?;
        self.queries
            .get_organization_by_id(id)
            .await?
            .ok_or_else(|| Dbpg001Error::NotFound(id.to_string()))
    }

    async fn get_organization_image(&self, id: Uuid) -> Result<StoredImage, Dbpg001Error> {
        self.read_image(ImageOwner::Organization, id).await
    }

    async fn set_company_image(&self, id: Uuid, bytes: Vec<u8>) -> Result<CompanyDto, Dbpg001Error> {
        self.set_image(ImageOwner::Company, id, bytes).await?;
        self.queries.get_company_by_id(id).await?.ok_or_else(|| Dbpg001Error::NotFound(id.to_string()))
    }

    async fn remove_company_image(&self, id: Uuid) -> Result<CompanyDto, Dbpg001Error> {
        self.remove_image(ImageOwner::Company, id).await?;
        self.queries.get_company_by_id(id).await?.ok_or_else(|| Dbpg001Error::NotFound(id.to_string()))
    }

    async fn get_company_image(&self, id: Uuid) -> Result<StoredImage, Dbpg001Error> {
        self.read_image(ImageOwner::Company, id).await
    }
}
