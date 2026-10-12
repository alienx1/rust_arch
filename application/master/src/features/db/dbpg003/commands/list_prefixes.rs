//! DBPG003 read: name prefixes from `db_master_lookup`.

use crate::features::db::dbpg003::commands::dbpg003_executor::Dbpg003CommandExecutor;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::PrefixOptionDto;

impl Dbpg003CommandExecutor {
    pub(super) async fn list_prefixes(&self) -> Result<Vec<PrefixOptionDto>, Dbpg003Error> {
        self.queries.list_prefixes().await
    }
}
