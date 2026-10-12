//! Sea-ORM query runner for employees: routes each `Dbpg003Queries` method to its file.
//!
//! Filtering, searching and paging run in SQL; labels and organizational names are loaded only
//! for the page being returned.

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::{
    BranchOptionDto, CompanyOptionDto, Dbpg003SummaryDto, DepartmentOptionDto, EmployeeDto,
    EmployeeFilterQuery, EmployeePageDto, PrefixOptionDto,
};

pub(crate) use super::like_search::{contains_ci, escape_like};

#[derive(Clone)]
pub struct DbDbpg003Queries {
    pub(super) db: DatabaseConnection,
}

impl DbDbpg003Queries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl super::Dbpg003Queries for DbDbpg003Queries {
    async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg003Error> {
        self.list_company_options().await
    }

    async fn list_branch_options(&self) -> Result<Vec<BranchOptionDto>, Dbpg003Error> {
        self.list_branch_options().await
    }

    async fn list_department_options(&self) -> Result<Vec<DepartmentOptionDto>, Dbpg003Error> {
        self.list_department_options().await
    }

    async fn list_prefixes(&self) -> Result<Vec<PrefixOptionDto>, Dbpg003Error> {
        self.list_prefixes().await
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
        self.get_summary().await
    }
}
