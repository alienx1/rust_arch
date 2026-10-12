//! Organization & Company Queries Trait and Implementation

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::db::dbpg000::errors::Dbpg000Error;
use crate::features::db::dbpg000::models::{
    CompanyDto, CompanyFilterQuery, CompanyTypeLookupDto, OrganizationDto, OrganizationSummaryDto,
};

pub mod db_dbpg000_queries;
mod get_company;
mod get_organization;
mod get_organization_summary;
mod list_companies;
mod list_company_types;
mod list_organizations;

pub use db_dbpg000_queries::DbDbpg000Queries;

#[async_trait]
pub trait Dbpg000Queries: Send + Sync {
    /// List all organizations with localized names and company count
    async fn list_organizations(&self) -> Result<Vec<OrganizationDto>, Dbpg000Error>;

    /// Retrieve a single organization by ID
    async fn get_organization_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<OrganizationDto>, Dbpg000Error>;

    /// List companies with optional filtering by organization, search query, active status
    async fn list_companies(
        &self,
        filter: CompanyFilterQuery,
    ) -> Result<Vec<CompanyDto>, Dbpg000Error>;

    /// Retrieve a single company by ID
    async fn get_company_by_id(&self, id: Uuid) -> Result<Option<CompanyDto>, Dbpg000Error>;

    /// List all company types (e.g. PUBLIC, PRIVATE) from db_master_lookup
    async fn list_company_types(&self) -> Result<Vec<CompanyTypeLookupDto>, Dbpg000Error>;

    /// Retrieve KPI summary of organizations and companies
    async fn get_summary(&self) -> Result<OrganizationSummaryDto, Dbpg000Error>;
}
