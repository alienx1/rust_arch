//! DBPG003 query: companies for the selectors / filter (delegates to DBPG002)

use crate::features::db::dbpg002::queries::Dbpg002Queries;
use crate::features::db::dbpg002::queries::db_dbpg002_queries::DbDbpg002Queries;

use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::CompanyOptionDto;
use crate::features::db::dbpg003::queries::db_dbpg003_queries::DbDbpg003Queries;

impl DbDbpg003Queries {
    pub(super) async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg003Error> {
        DbDbpg002Queries::new(self.db.clone())
            .list_company_options()
            .await
            .map_err(|e| Dbpg003Error::Database(e.to_string()))
    }
}
