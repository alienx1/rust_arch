//! Sea-ORM query runner for branches and departments: routes each `Dbpg002Queries` method to its file

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{
    BranchDto, BranchFilterQuery, BranchTypeLookupDto, PermissionGroupOptionDto, CompanyOptionDto, Dbpg002SummaryDto,
    DepartmentDto, DepartmentFilterQuery,
};

pub(crate) use super::label_lookup::{Names, fold_names};

/// Direct Sea-ORM query runner for branch and department master data.
#[derive(Clone)]
pub struct DbDbpg002Queries {
    pub(super) db: DatabaseConnection,
}

impl DbDbpg002Queries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl super::Dbpg002Queries for DbDbpg002Queries {
    async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg002Error> {
        self.list_company_options().await
    }

    async fn list_permission_group_options(&self) -> Result<Vec<PermissionGroupOptionDto>, Dbpg002Error> {
        self.list_permission_group_options().await
    }

    async fn list_branch_types(&self) -> Result<Vec<BranchTypeLookupDto>, Dbpg002Error> {
        self.list_branch_types().await
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
        self.get_summary().await
    }
}
