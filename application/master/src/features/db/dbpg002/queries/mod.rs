//! DBPG002 Queries trait and Sea-ORM implementation

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{
    BranchDto, BranchFilterQuery, BranchTypeLookupDto, CompanyOptionDto, PermissionGroupOptionDto, Dbpg002SummaryDto,
    DepartmentDto, DepartmentFilterQuery,
};

mod branch_dto_assembly;
pub mod db_dbpg002_queries;
mod department_dto_assembly;
mod get_branch_by_id;
mod get_department_by_id;
mod get_summary;
mod label_lookup;
mod list_branch_types;
mod list_branches;
mod list_company_options;
mod list_permission_group_options;
mod list_departments;

pub use db_dbpg002_queries::DbDbpg002Queries;

#[async_trait]
pub trait Dbpg002Queries: Send + Sync {
    /// Companies for the company filter / parent selector
    async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg002Error>;

    /// Active permission groups, for a department's default group
    async fn list_permission_group_options(&self) -> Result<Vec<PermissionGroupOptionDto>, Dbpg002Error>;

    /// Branch types (HEAD / BRANCH) from `db_master_lookup`
    async fn list_branch_types(&self) -> Result<Vec<BranchTypeLookupDto>, Dbpg002Error>;

    /// List branches with optional company / search / active filters
    async fn list_branches(
        &self,
        filter: BranchFilterQuery,
    ) -> Result<Vec<BranchDto>, Dbpg002Error>;

    /// Retrieve a single branch by ID
    async fn get_branch_by_id(&self, id: Uuid) -> Result<Option<BranchDto>, Dbpg002Error>;

    /// List departments with optional company / branch / search / active filters
    async fn list_departments(
        &self,
        filter: DepartmentFilterQuery,
    ) -> Result<Vec<DepartmentDto>, Dbpg002Error>;

    /// Retrieve a single department by ID
    async fn get_department_by_id(&self, id: Uuid) -> Result<Option<DepartmentDto>, Dbpg002Error>;

    /// KPI summary of branches and departments
    async fn get_summary(&self) -> Result<Dbpg002SummaryDto, Dbpg002Error>;
}
