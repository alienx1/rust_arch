//! DBPG001 lookups backed by the DBPG000 Sea-ORM query runner (same tables, same DTOs)

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::features::db::dbpg000::queries::Dbpg000Queries;
use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::{CompanyDto, CompanyTypeLookupDto, OrganizationDto};

#[derive(Clone)]
pub struct DbDbpg001Queries {
    pub(super) reads: Arc<dyn Dbpg000Queries>,
}

impl DbDbpg001Queries {
    pub fn new(reads: Arc<dyn Dbpg000Queries>) -> Self {
        Self { reads }
    }
}

#[async_trait]
impl super::Dbpg001Queries for DbDbpg001Queries {
    async fn list_organizations(&self) -> Result<Vec<OrganizationDto>, Dbpg001Error> {
        self.list_organizations().await
    }

    async fn get_organization_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<OrganizationDto>, Dbpg001Error> {
        self.get_organization_by_id(id).await
    }

    async fn get_company_by_id(&self, id: Uuid) -> Result<Option<CompanyDto>, Dbpg001Error> {
        self.get_company_by_id(id).await
    }

    async fn list_company_types(&self) -> Result<Vec<CompanyTypeLookupDto>, Dbpg001Error> {
        self.list_company_types().await
    }
}
