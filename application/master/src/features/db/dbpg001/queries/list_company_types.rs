//! DBPG001 query: list company types for the type selector

use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::CompanyTypeLookupDto;

use super::db_dbpg001_queries::DbDbpg001Queries;

impl DbDbpg001Queries {
    pub(super) async fn list_company_types(
        &self,
    ) -> Result<Vec<CompanyTypeLookupDto>, Dbpg001Error> {
        Ok(self.reads.list_company_types().await?)
    }
}
