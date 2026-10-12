//! DBPG000 command: list all organizations with localized names and company count

use crate::features::db::dbpg000::errors::Dbpg000Error;
use crate::features::db::dbpg000::models::OrganizationDto;

use super::dbpg000_executor::Dbpg000CommandExecutor;

impl Dbpg000CommandExecutor {
    pub(super) async fn list_organizations(&self) -> Result<Vec<OrganizationDto>, Dbpg000Error> {
        self.queries.list_organizations().await
    }
}
