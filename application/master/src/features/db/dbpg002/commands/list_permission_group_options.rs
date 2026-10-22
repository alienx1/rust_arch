//! DBPG002 read: permission groups for a department's default group.

use crate::features::db::dbpg002::commands::dbpg002_executor::Dbpg002CommandExecutor;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::PermissionGroupOptionDto;

impl Dbpg002CommandExecutor {
    pub(super) async fn list_permission_group_options(&self) -> Result<Vec<PermissionGroupOptionDto>, Dbpg002Error> {
        self.queries.list_permission_group_options().await
    }
}
