//! DBPG000 command: list all company types (e.g. PUBLIC, PRIVATE) from db_master_lookup

use crate::features::db::dbpg000::errors::Dbpg000Error;
use crate::features::db::dbpg000::models::CompanyTypeLookupDto;

use super::dbpg000_executor::Dbpg000CommandExecutor;

impl Dbpg000CommandExecutor {
    pub(super) async fn list_company_types(
        &self,
    ) -> Result<Vec<CompanyTypeLookupDto>, Dbpg000Error> {
        self.queries.list_company_types().await
    }
}
