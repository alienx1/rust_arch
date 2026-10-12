//! DBPG001 command: list company types for the type selector

use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::CompanyTypeLookupDto;

use super::dbpg001_executor::Dbpg001CommandExecutor;

impl Dbpg001CommandExecutor {
    pub(super) async fn list_company_types(
        &self,
    ) -> Result<Vec<CompanyTypeLookupDto>, Dbpg001Error> {
        self.queries.list_company_types().await
    }
}
