//! DBPG002 Command Executor: routes each `Dbpg002Commands` method to its action file.
//!
//! Reads delegate to `Dbpg002Queries`. Each write runs in one transaction so a branch never
//! exists without its address/contact groups or labels.

use application_core::traits::{ReadCaches, cache_ns, cache_ttl};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

use crate::features::db::dbpg002::commands::Dbpg002Commands;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{
    BranchDto, BranchFilterQuery, BranchTypeLookupDto, CompanyOptionDto, PermissionGroupOptionDto, CreateBranchCommand,
    CreateDepartmentCommand, Dbpg002SummaryDto, DepartmentDto, DepartmentFilterQuery,
    UpdateBranchCommand, UpdateDepartmentCommand,
};
use crate::features::db::dbpg002::queries::Dbpg002Queries;

pub struct Dbpg002CommandExecutor {
    pub(super) queries: Arc<dyn Dbpg002Queries>,
    pub(super) db: DatabaseConnection,
    /// Valkey read-through for the rarely changing reads (off until `with_cache`)
    pub(super) caches: ReadCaches,
}

impl Dbpg002CommandExecutor {
    pub fn new(queries: Arc<dyn Dbpg002Queries>, db: DatabaseConnection) -> Self {
        Self { queries, db, caches: ReadCaches::disabled() }
    }

    /// Reads that change rarely are served from Valkey; writes clear them (see the cache-invalidation middleware)
    pub fn with_cache(mut self, caches: ReadCaches) -> Self {
        self.caches = caches;
        self
    }
}

#[async_trait]
impl Dbpg002Commands for Dbpg002CommandExecutor {
    async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg002Error> {
        let cache = self.caches.ns(cache_ns::ORG);
        let key = cache.key("list_company_options", &());
        cache.through_key(key, cache_ttl::ORG, || self.list_company_options()).await
    }

    async fn list_permission_group_options(&self) -> Result<Vec<PermissionGroupOptionDto>, Dbpg002Error> {
        self.list_permission_group_options().await
    }

    async fn list_branch_types(&self) -> Result<Vec<BranchTypeLookupDto>, Dbpg002Error> {
        let cache = self.caches.ns(cache_ns::REF);
        let key = cache.key("list_branch_types", &());
        cache.through_key(key, cache_ttl::REF, || self.list_branch_types()).await
    }

    async fn list_branches(
        &self,
        filter: BranchFilterQuery,
    ) -> Result<Vec<BranchDto>, Dbpg002Error> {
        self.list_branches(filter).await
    }

    async fn get_branch_by_id(&self, id: Uuid) -> Result<Option<BranchDto>, Dbpg002Error> {
        self.get_branch_by_id(id).await
    }

    async fn list_departments(
        &self,
        filter: DepartmentFilterQuery,
    ) -> Result<Vec<DepartmentDto>, Dbpg002Error> {
        self.list_departments(filter).await
    }

    async fn get_department_by_id(&self, id: Uuid) -> Result<Option<DepartmentDto>, Dbpg002Error> {
        self.get_department_by_id(id).await
    }

    async fn get_summary(&self) -> Result<Dbpg002SummaryDto, Dbpg002Error> {
        let cache = self.caches.ns(cache_ns::ORG);
        let key = cache.key("get_summary", &());
        cache.through_key(key, cache_ttl::ORG, || self.get_summary()).await
    }

    async fn create_branch(
        &self,
        cmd: CreateBranchCommand,
        actor_id: Option<Uuid>,
    ) -> Result<BranchDto, Dbpg002Error> {
        self.create_branch(cmd, actor_id).await
    }

    async fn update_branch(
        &self,
        id: Uuid,
        cmd: UpdateBranchCommand,
        actor_id: Option<Uuid>,
    ) -> Result<BranchDto, Dbpg002Error> {
        self.update_branch(id, cmd, actor_id).await
    }

    async fn create_department(
        &self,
        cmd: CreateDepartmentCommand,
        actor_id: Option<Uuid>,
    ) -> Result<DepartmentDto, Dbpg002Error> {
        self.create_department(cmd, actor_id).await
    }

    async fn update_department(
        &self,
        id: Uuid,
        cmd: UpdateDepartmentCommand,
        actor_id: Option<Uuid>,
    ) -> Result<DepartmentDto, Dbpg002Error> {
        self.update_department(id, cmd, actor_id).await
    }

    async fn delete_branch(&self, id: Uuid) -> Result<(), Dbpg002Error> {
        self.delete_branch(id).await
    }

    async fn delete_department(&self, id: Uuid) -> Result<(), Dbpg002Error> {
        self.delete_department(id).await
    }
}
