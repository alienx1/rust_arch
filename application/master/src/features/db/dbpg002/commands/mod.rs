//! DBPG002 Commands trait (reads + writes consumed by routers) and executor

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{
    BranchDto, BranchFilterQuery, BranchTypeLookupDto, CompanyOptionDto, CreateBranchCommand,
    CreateDepartmentCommand, Dbpg002SummaryDto, DepartmentDto, DepartmentFilterQuery,
    UpdateBranchCommand, UpdateDepartmentCommand,
};

mod branch_label_writer;
mod branch_type_rules;
mod create_branch;
mod create_department;
mod delete_branch;
mod delete_department;
pub mod dbpg002_executor;
mod department_label_writer;
mod get_branch_by_id;
mod get_department_by_id;
mod get_summary;
mod list_branch_types;
mod list_branches;
mod list_company_options;
mod list_departments;
mod name_rules;
mod update_branch;
mod update_department;

pub use dbpg002_executor::Dbpg002CommandExecutor;

#[async_trait]
pub trait Dbpg002Commands: Send + Sync {
    /// Companies for the company filter / parent selector
    async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg002Error>;

    /// Branch types (HEAD / BRANCH)
    async fn list_branch_types(&self) -> Result<Vec<BranchTypeLookupDto>, Dbpg002Error>;

    /// List branches with optional filters
    async fn list_branches(
        &self,
        filter: BranchFilterQuery,
    ) -> Result<Vec<BranchDto>, Dbpg002Error>;

    /// Retrieve a single branch by ID
    async fn get_branch_by_id(&self, id: Uuid) -> Result<Option<BranchDto>, Dbpg002Error>;

    /// List departments with optional filters
    async fn list_departments(
        &self,
        filter: DepartmentFilterQuery,
    ) -> Result<Vec<DepartmentDto>, Dbpg002Error>;

    /// Retrieve a single department by ID
    async fn get_department_by_id(&self, id: Uuid) -> Result<Option<DepartmentDto>, Dbpg002Error>;

    /// KPI summary of branches and departments
    async fn get_summary(&self) -> Result<Dbpg002SummaryDto, Dbpg002Error>;

    /// Create a branch (with its address/contact groups); one headquarter per company
    async fn create_branch(
        &self,
        cmd: CreateBranchCommand,
        actor_id: Option<Uuid>,
    ) -> Result<BranchDto, Dbpg002Error>;

    /// Update a branch with optimistic lock verification
    async fn update_branch(
        &self,
        id: Uuid,
        cmd: UpdateBranchCommand,
        actor_id: Option<Uuid>,
    ) -> Result<BranchDto, Dbpg002Error>;

    /// Create a department under a branch
    async fn create_department(
        &self,
        cmd: CreateDepartmentCommand,
        actor_id: Option<Uuid>,
    ) -> Result<DepartmentDto, Dbpg002Error>;

    /// Update a department (optionally moving it to another branch) with optimistic lock
    async fn update_department(
        &self,
        id: Uuid,
        cmd: UpdateDepartmentCommand,
        actor_id: Option<Uuid>,
    ) -> Result<DepartmentDto, Dbpg002Error>;

    /// Hard-delete a branch with its labels and address/contact groups; refused for the headquarter
    /// and, with `InUse`, while other records still reference it
    async fn delete_branch(&self, id: Uuid) -> Result<(), Dbpg002Error>;

    /// Hard-delete a department with its labels; `InUse` while other records still reference it
    async fn delete_department(&self, id: Uuid) -> Result<(), Dbpg002Error>;
}
