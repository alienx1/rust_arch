//! DBPG003 Commands trait (reads + writes consumed by routers) and executor

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::{
    BranchOptionDto, CompanyOptionDto, CreateEmployeeCommand, Dbpg003SummaryDto,
    DepartmentOptionDto, EmployeeDto, EmployeeFilterQuery, EmployeePageDto, PrefixOptionDto,
    StoredImage, UpdateEmployeeCommand,
};

mod create_employee;
mod delete_employee;
pub mod dbpg003_executor;
mod employee_image;
mod employee_input_rules;
mod employee_reference_rules;
mod get_employee_by_id;
mod get_summary;
mod list_branch_options;
mod list_company_options;
mod list_department_options;
mod list_employees;
mod list_prefixes;
mod person_name_writer;
mod reactivate_employee_accounts;
mod suspend_employee_accounts;
mod update_employee;

pub use dbpg003_executor::Dbpg003CommandExecutor;

#[async_trait]
pub trait Dbpg003Commands: Send + Sync {
    async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg003Error>;
    async fn list_branch_options(&self) -> Result<Vec<BranchOptionDto>, Dbpg003Error>;
    async fn list_department_options(&self) -> Result<Vec<DepartmentOptionDto>, Dbpg003Error>;
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

    /// Create an employee with its person name (one transaction)
    async fn create_employee(
        &self,
        cmd: CreateEmployeeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<EmployeeDto, Dbpg003Error>;

    /// Update an employee with optimistic lock verification
    async fn update_employee(
        &self,
        id: Uuid,
        cmd: UpdateEmployeeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<EmployeeDto, Dbpg003Error>;

    /// Delete an employee with its person name and labels; refused with `InUse` while a user
    /// account (or anything else) still references it
    async fn delete_employee(&self, id: Uuid) -> Result<(), Dbpg003Error>;

    /// Replaces the employee's photo (PNG / JPEG / GIF / WebP, at most 2 MB, judged by content)
    async fn set_employee_image(&self, _id: Uuid, _bytes: Vec<u8>) -> Result<EmployeeDto, Dbpg003Error> {
        Err(Dbpg003Error::Internal("Image storage is not configured".to_string()))
    }

    /// Removes the employee's photo
    async fn remove_employee_image(&self, _id: Uuid) -> Result<EmployeeDto, Dbpg003Error> {
        Err(Dbpg003Error::Internal("Image storage is not configured".to_string()))
    }

    /// The employee's photo bytes; `NotFound` when there is none
    async fn get_employee_image(&self, _id: Uuid) -> Result<StoredImage, Dbpg003Error> {
        Err(Dbpg003Error::Internal("Image storage is not configured".to_string()))
    }
}
