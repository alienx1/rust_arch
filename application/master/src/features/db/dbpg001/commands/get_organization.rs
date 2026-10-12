//! DBPG001 command: retrieve the organization to edit

use uuid::Uuid;

use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::OrganizationDto;

use super::dbpg001_executor::Dbpg001CommandExecutor;

impl Dbpg001CommandExecutor {
    pub(super) async fn get_organization_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<OrganizationDto>, Dbpg001Error> {
        self.queries.get_organization_by_id(id).await
    }
}
