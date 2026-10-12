//! DBPG001 command: list organizations for the parent selector

use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::OrganizationDto;

use super::dbpg001_executor::Dbpg001CommandExecutor;

impl Dbpg001CommandExecutor {
    pub(super) async fn list_organizations(&self) -> Result<Vec<OrganizationDto>, Dbpg001Error> {
        self.queries.list_organizations().await
    }
}
