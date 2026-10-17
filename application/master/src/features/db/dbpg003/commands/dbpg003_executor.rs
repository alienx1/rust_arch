//! DBPG003 Command Executor: routes each `Dbpg003Commands` method to its action file.
//!
//! Reads delegate to `Dbpg003Queries`. An employee and its person name (`db_person_name` +
//! labels) are written in one transaction. Organization, company and branch come from the
//! chosen department, so they can never disagree with it.

use application_core::traits::{ReadCaches, cache_ns, cache_ttl};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

use crate::features::db::dbpg003::commands::Dbpg003Commands;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::{
    BranchOptionDto, CompanyOptionDto, CreateEmployeeCommand, Dbpg003SummaryDto,
    DepartmentOptionDto, EmployeeDto, EmployeeFilterQuery, EmployeePageDto, PrefixOptionDto,
    StoredImage, UpdateEmployeeCommand,
};
use crate::features::db::dbpg003::queries::Dbpg003Queries;
use crate::features::db::image_object_store::ImageObjectStore;
use crate::features::su::supg005::Supg005Commands;

pub struct Dbpg003CommandExecutor {
    pub(super) queries: Arc<dyn Dbpg003Queries>,
    pub(super) db: DatabaseConnection,
    /// Suspends the user accounts of an employee that is switched off (identity + local flag)
    pub(super) accounts: Option<Arc<dyn Supg005Commands>>,
    /// Valkey read-through for the rarely changing reads (off until `with_cache`)
    pub(super) caches: ReadCaches,
    /// Object storage for the photos (images are refused until `with_images`)
    pub(super) images: Option<ImageObjectStore>,
}

impl Dbpg003CommandExecutor {
    pub fn new(queries: Arc<dyn Dbpg003Queries>, db: DatabaseConnection) -> Self {
        Self { queries, db, accounts: None, caches: ReadCaches::disabled(), images: None }
    }

    /// Reads that change rarely are served from Valkey; writes clear them (see the cache-invalidation middleware)
    pub fn with_cache(mut self, caches: ReadCaches) -> Self {
        self.caches = caches;
        self
    }

    /// Employee photos are kept in `bucket` of `content`
    pub fn with_images(mut self, content: Arc<dyn application_core::traits::ContentService>, bucket: impl Into<String>) -> Self {
        self.images = Some(ImageObjectStore::new(content, bucket));
        self
    }

    /// Attaches the account-suspension port: switching an employee off then suspends the user
    /// accounts linked to it
    pub fn with_account_suspension(mut self, accounts: Arc<dyn Supg005Commands>) -> Self {
        self.accounts = Some(accounts);
        self
    }
}

#[async_trait]
impl Dbpg003Commands for Dbpg003CommandExecutor {
    async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg003Error> {
        let cache = self.caches.ns(cache_ns::ORG);
        let key = cache.key("list_company_options", &());
        cache.through_key(key, cache_ttl::ORG, || self.list_company_options()).await
    }

    async fn list_branch_options(&self) -> Result<Vec<BranchOptionDto>, Dbpg003Error> {
        let cache = self.caches.ns(cache_ns::ORG);
        let key = cache.key("list_branch_options", &());
        cache.through_key(key, cache_ttl::ORG, || self.list_branch_options()).await
    }

    async fn list_department_options(&self) -> Result<Vec<DepartmentOptionDto>, Dbpg003Error> {
        let cache = self.caches.ns(cache_ns::ORG);
        let key = cache.key("list_department_options", &());
        cache.through_key(key, cache_ttl::ORG, || self.list_department_options()).await
    }

    async fn list_prefixes(&self) -> Result<Vec<PrefixOptionDto>, Dbpg003Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("list_prefixes", &());
        cache.through_key(key, cache_ttl::REF, || self.list_prefixes()).await
    }

    async fn list_employees(
        &self,
        filter: EmployeeFilterQuery,
    ) -> Result<EmployeePageDto, Dbpg003Error> {
        self.list_employees(filter).await
    }

    async fn get_employee_by_id(&self, id: Uuid) -> Result<Option<EmployeeDto>, Dbpg003Error> {
        self.get_employee_by_id(id).await
    }

    async fn get_summary(&self) -> Result<Dbpg003SummaryDto, Dbpg003Error> {
        let cache = self.caches.ns(cache_ns::ORG);
        let key = cache.key("get_summary", &());
        cache.through_key(key, cache_ttl::ORG, || self.get_summary()).await
    }

    async fn create_employee(
        &self,
        cmd: CreateEmployeeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<EmployeeDto, Dbpg003Error> {
        self.create_employee(cmd, actor_id).await
    }

    async fn update_employee(
        &self,
        id: Uuid,
        cmd: UpdateEmployeeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<EmployeeDto, Dbpg003Error> {
        self.update_employee(id, cmd, actor_id).await
    }

    async fn delete_employee(&self, id: Uuid) -> Result<(), Dbpg003Error> {
        self.delete_employee(id).await
    }

    async fn set_employee_image(&self, id: Uuid, bytes: Vec<u8>) -> Result<EmployeeDto, Dbpg003Error> {
        self.set_image(id, bytes).await?;
        self.queries.get_employee_by_id(id).await?.ok_or_else(|| Dbpg003Error::NotFound(id.to_string()))
    }

    async fn remove_employee_image(&self, id: Uuid) -> Result<EmployeeDto, Dbpg003Error> {
        self.remove_image(id).await?;
        self.queries.get_employee_by_id(id).await?.ok_or_else(|| Dbpg003Error::NotFound(id.to_string()))
    }

    async fn get_employee_image(&self, id: Uuid) -> Result<StoredImage, Dbpg003Error> {
        self.read_image(id).await
    }
}
