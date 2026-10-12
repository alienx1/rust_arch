//! DBPG000 command: list companies with optional filtering by organization, search query, active status

use crate::features::db::dbpg000::errors::Dbpg000Error;
use crate::features::db::dbpg000::models::{CompanyDto, CompanyFilterQuery};

use super::dbpg000_executor::Dbpg000CommandExecutor;

impl Dbpg000CommandExecutor {
    pub(super) async fn list_companies(
        &self,
        filter: CompanyFilterQuery,
    ) -> Result<Vec<CompanyDto>, Dbpg000Error> {
        self.queries.list_companies(filter).await
    }
}
