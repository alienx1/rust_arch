//! DBPG004 command: list one page of lookups matching the filter

use crate::features::db::dbpg004::errors::Dbpg004Error;
use crate::features::db::dbpg004::models::{LookupFilterQuery, LookupPageDto};

use super::dbpg004_executor::Dbpg004CommandExecutor;

impl Dbpg004CommandExecutor {
    pub(super) async fn list_lookups(
        &self,
        filter: LookupFilterQuery,
    ) -> Result<LookupPageDto, Dbpg004Error> {
        self.queries.list_lookups(filter).await
    }
}
