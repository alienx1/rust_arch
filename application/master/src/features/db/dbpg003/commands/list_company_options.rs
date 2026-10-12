//! DBPG003 read: companies for the selectors / filter.

use crate::features::db::dbpg003::commands::dbpg003_executor::Dbpg003CommandExecutor;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::CompanyOptionDto;

impl Dbpg003CommandExecutor {
    pub(super) async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg003Error> {
        self.queries.list_company_options().await
    }
}
