//! DBPG003 read: all branches (filtered client-side by company).

use crate::features::db::dbpg003::commands::dbpg003_executor::Dbpg003CommandExecutor;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::BranchOptionDto;

impl Dbpg003CommandExecutor {
    pub(super) async fn list_branch_options(&self) -> Result<Vec<BranchOptionDto>, Dbpg003Error> {
        self.queries.list_branch_options().await
    }
}
