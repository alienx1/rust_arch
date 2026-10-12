//! DBPG001 query: retrieve the company to edit

use uuid::Uuid;

use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::CompanyDto;

use super::db_dbpg001_queries::DbDbpg001Queries;

impl DbDbpg001Queries {
    pub(super) async fn get_company_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<CompanyDto>, Dbpg001Error> {
        Ok(self.reads.get_company_by_id(id).await?)
    }
}
