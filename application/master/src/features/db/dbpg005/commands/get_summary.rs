//! Get the per-level row counts (delegates to `Dbpg005Queries`)

use super::dbpg005_executor::Dbpg005CommandExecutor;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::Dbpg005SummaryDto;

impl Dbpg005CommandExecutor {
    pub(super) async fn get_summary(&self) -> Result<Dbpg005SummaryDto, Dbpg005Error> {
        self.queries.get_summary().await
    }
}
