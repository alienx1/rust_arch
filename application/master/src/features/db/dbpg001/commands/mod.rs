//! DBPG001 Commands trait (writes + form lookups consumed by routers) and executor

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::{
    CompanyDto, CompanyTypeLookupDto, CreateCompanyCommand, CreateOrganizationCommand, OrganizationDto,
    StoredImage, UpdateCompanyCommand, UpdateOrganizationCommand,
};

pub mod dbpg001_executor;
mod create_company;
mod create_organization;
mod delete_company;
mod entity_image;
mod delete_organization;
mod get_company;
mod get_organization;
mod list_company_types;
mod list_organizations;
mod update_company;
mod update_organization;

pub use dbpg001_executor::Dbpg001CommandExecutor;

#[async_trait]
pub trait Dbpg001Commands: Send + Sync {
    /// Organizations for the parent selector
    async fn list_organizations(&self) -> Result<Vec<OrganizationDto>, Dbpg001Error>;

    /// Organization to edit
    async fn get_organization_by_id(&self, id: Uuid)
    -> Result<Option<OrganizationDto>, Dbpg001Error>;

    /// Company to edit
    async fn get_company_by_id(&self, id: Uuid) -> Result<Option<CompanyDto>, Dbpg001Error>;

    /// Company types for the type selector
    async fn list_company_types(&self) -> Result<Vec<CompanyTypeLookupDto>, Dbpg001Error>;

    /// Create a new organization
    async fn create_organization(
        &self,
        cmd: CreateOrganizationCommand,
        actor_id: Option<Uuid>,
    ) -> Result<OrganizationDto, Dbpg001Error>;

    /// Update an existing organization with optimistic lock verification
    async fn update_organization(
        &self,
        id: Uuid,
        cmd: UpdateOrganizationCommand,
        actor_id: Option<Uuid>,
    ) -> Result<OrganizationDto, Dbpg001Error>;

    /// Create a new company under an organization
    async fn create_company(
        &self,
        cmd: CreateCompanyCommand,
        actor_id: Option<Uuid>,
    ) -> Result<CompanyDto, Dbpg001Error>;

    /// Update an existing company with optimistic lock verification
    async fn update_company(
        &self,
        id: Uuid,
        cmd: UpdateCompanyCommand,
        actor_id: Option<Uuid>,
    ) -> Result<CompanyDto, Dbpg001Error>;

    /// Delete an organization (and its labels) when nothing references it
    async fn delete_organization(&self, id: Uuid) -> Result<(), Dbpg001Error>;

    /// Delete a company (and its labels) when nothing references it
    async fn delete_company(&self, id: Uuid) -> Result<(), Dbpg001Error>;

    /// Replaces the organization's logo (PNG / JPEG / GIF / WebP, at most 2 MB, judged by content)
    async fn set_organization_image(&self, _id: Uuid, _bytes: Vec<u8>) -> Result<OrganizationDto, Dbpg001Error> {
        Err(Dbpg001Error::Internal("Image storage is not configured".to_string()))
    }

    /// Removes the organization's logo
    async fn remove_organization_image(&self, _id: Uuid) -> Result<OrganizationDto, Dbpg001Error> {
        Err(Dbpg001Error::Internal("Image storage is not configured".to_string()))
    }

    /// The organization's logo bytes; `NotFound` when it has none
    async fn get_organization_image(&self, _id: Uuid) -> Result<StoredImage, Dbpg001Error> {
        Err(Dbpg001Error::Internal("Image storage is not configured".to_string()))
    }

    /// Replaces the company's logo (same rules as the organization's)
    async fn set_company_image(&self, _id: Uuid, _bytes: Vec<u8>) -> Result<CompanyDto, Dbpg001Error> {
        Err(Dbpg001Error::Internal("Image storage is not configured".to_string()))
    }

    /// Removes the company's logo
    async fn remove_company_image(&self, _id: Uuid) -> Result<CompanyDto, Dbpg001Error> {
        Err(Dbpg001Error::Internal("Image storage is not configured".to_string()))
    }

    /// The company's logo bytes; `NotFound` when it has none
    async fn get_company_image(&self, _id: Uuid) -> Result<StoredImage, Dbpg001Error> {
        Err(Dbpg001Error::Internal("Image storage is not configured".to_string()))
    }
}
