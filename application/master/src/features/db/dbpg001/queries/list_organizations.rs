//! DBPG001 query: list organizations for the parent selector

use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::OrganizationDto;

use super::db_dbpg001_queries::DbDbpg001Queries;

impl DbDbpg001Queries {
    pub(super) async fn list_organizations(&self) -> Result<Vec<OrganizationDto>, Dbpg001Error> {
        Ok(self.reads.list_organizations().await?)
    }
}
