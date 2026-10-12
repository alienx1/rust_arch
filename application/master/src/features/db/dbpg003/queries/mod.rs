//! DBPG003 Queries trait and Sea-ORM implementation

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::{
    BranchOptionDto, CompanyOptionDto, Dbpg003SummaryDto, DepartmentOptionDto, EmployeeDto,
    EmployeeFilterQuery, EmployeePageDto, PrefixOptionDto,
};

pub mod db_dbpg003_queries;
mod employee_dto_assembly;
mod get_employee_by_id;
mod get_summary;
mod like_search;
mod list_branch_options;
mod list_company_options;
mod list_department_options;
mod list_employees;
mod list_prefixes;
pub use db_dbpg003_queries::DbDbpg003Queries;

#[async_trait]
pub trait Dbpg003Queries: Send + Sync {
    /// Companies for the selectors / filter
    async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg003Error>;

    /// All branches (filtered client-side by company)
    async fn list_branch_options(&self) -> Result<Vec<BranchOptionDto>, Dbpg003Error>;

    /// All departments (filtered client-side by branch)
    async fn list_department_options(&self) -> Result<Vec<DepartmentOptionDto>, Dbpg003Error>;

    /// Name prefixes from `db_master_lookup` (category `prefix`)
    async fn list_prefixes(&self) -> Result<Vec<PrefixOptionDto>, Dbpg003Error>;

    /// One page of employees matching the filter
    async fn list_employees(
        &self,
        filter: EmployeeFilterQuery,
    ) -> Result<EmployeePageDto, Dbpg003Error>;

    /// Retrieve a single employee by ID
    async fn get_employee_by_id(&self, id: Uuid) -> Result<Option<EmployeeDto>, Dbpg003Error>;

    /// KPI summary of employees
    async fn get_summary(&self) -> Result<Dbpg003SummaryDto, Dbpg003Error>;
}
