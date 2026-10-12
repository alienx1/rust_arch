//! DBPG002 read: branches with optional filters.

use crate::features::db::dbpg002::commands::dbpg002_executor::Dbpg002CommandExecutor;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{BranchDto, BranchFilterQuery};

impl Dbpg002CommandExecutor {
    pub(super) async fn list_branches(
        &self,
        filter: BranchFilterQuery,
    ) -> Result<Vec<BranchDto>, Dbpg002Error> {
        self.queries.list_branches(filter).await
    }
}
