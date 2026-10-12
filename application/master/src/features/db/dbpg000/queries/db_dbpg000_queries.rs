//! Sea-ORM Query Runner for Organizations and Companies

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::features::db::dbpg000::errors::Dbpg000Error;
use crate::features::db::dbpg000::models::{
    CompanyDto, CompanyFilterQuery, CompanyTypeLookupDto, OrganizationDto, OrganizationSummaryDto,
};

/// Direct Sea-ORM query runner for organization and company master data.
#[derive(Clone)]
pub struct DbDbpg000Queries {
    pub(super) db: DatabaseConnection,
}

impl DbDbpg000Queries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Access database connection reference
    pub fn connection(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl super::Dbpg000Queries for DbDbpg000Queries {
    async fn list_organizations(&self) -> Result<Vec<OrganizationDto>, Dbpg000Error> {
        self.list_organizations().await
    }

    async fn get_organization_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<OrganizationDto>, Dbpg000Error> {
        self.get_organization_by_id(id).await
    }

    async fn list_companies(
        &self,
        filter: CompanyFilterQuery,
    ) -> Result<Vec<CompanyDto>, Dbpg000Error> {
        self.list_companies(filter).await
    }

    async fn get_company_by_id(&self, id: Uuid) -> Result<Option<CompanyDto>, Dbpg000Error> {
        self.get_company_by_id(id).await
    }

    async fn list_company_types(&self) -> Result<Vec<CompanyTypeLookupDto>, Dbpg000Error> {
        self.list_company_types().await
    }

    async fn get_summary(&self) -> Result<OrganizationSummaryDto, Dbpg000Error> {
        self.get_summary().await
    }
}
