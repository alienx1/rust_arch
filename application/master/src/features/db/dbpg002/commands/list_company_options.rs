//! DBPG002 read: companies for the company filter / parent selector.

use crate::features::db::dbpg002::commands::dbpg002_executor::Dbpg002CommandExecutor;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::CompanyOptionDto;

impl Dbpg002CommandExecutor {
    pub(super) async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg002Error> {
        self.queries.list_company_options().await
    }
}
