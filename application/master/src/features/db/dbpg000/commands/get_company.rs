//! DBPG000 command: retrieve a single company by ID

use uuid::Uuid;

use crate::features::db::dbpg000::errors::Dbpg000Error;
use crate::features::db::dbpg000::models::CompanyDto;

use super::dbpg000_executor::Dbpg000CommandExecutor;

impl Dbpg000CommandExecutor {
    pub(super) async fn get_company_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<CompanyDto>, Dbpg000Error> {
        self.queries.get_company_by_id(id).await
    }
}
