//! DBPG002 read: branch types (HEAD / BRANCH).

use crate::features::db::dbpg002::commands::dbpg002_executor::Dbpg002CommandExecutor;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::BranchTypeLookupDto;

impl Dbpg002CommandExecutor {
    pub(super) async fn list_branch_types(&self) -> Result<Vec<BranchTypeLookupDto>, Dbpg002Error> {
        self.queries.list_branch_types().await
    }
}
