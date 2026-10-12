//! DBPG001 form lookups (parent organizations, company types, record to edit)

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::{CompanyDto, CompanyTypeLookupDto, OrganizationDto};

pub mod db_dbpg001_queries;
mod get_company;
mod get_organization;
mod list_company_types;
mod list_organizations;

pub use db_dbpg001_queries::DbDbpg001Queries;

#[async_trait]
pub trait Dbpg001Queries: Send + Sync {
    /// Organizations for the parent selector and the edit form
    async fn list_organizations(&self) -> Result<Vec<OrganizationDto>, Dbpg001Error>;

    /// Organization to edit
    async fn get_organization_by_id(&self, id: Uuid)
    -> Result<Option<OrganizationDto>, Dbpg001Error>;

    /// Company to edit
    async fn get_company_by_id(&self, id: Uuid) -> Result<Option<CompanyDto>, Dbpg001Error>;

    /// Company types for the type selector
    async fn list_company_types(&self) -> Result<Vec<CompanyTypeLookupDto>, Dbpg001Error>;
}
