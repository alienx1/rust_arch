//! DBPG001 command: retrieve the company to edit

use uuid::Uuid;

use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::CompanyDto;

use super::dbpg001_executor::Dbpg001CommandExecutor;

impl Dbpg001CommandExecutor {
    pub(super) async fn get_company_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<CompanyDto>, Dbpg001Error> {
        self.queries.get_company_by_id(id).await
    }
}
