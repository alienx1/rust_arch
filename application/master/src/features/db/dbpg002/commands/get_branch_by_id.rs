//! DBPG002 read: a single branch by id.

use uuid::Uuid;

use crate::features::db::dbpg002::commands::dbpg002_executor::Dbpg002CommandExecutor;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::BranchDto;

impl Dbpg002CommandExecutor {
    pub(super) async fn get_branch_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<BranchDto>, Dbpg002Error> {
        self.queries.get_branch_by_id(id).await
    }
}
