//! DBPG002 read: KPI summary of branches and departments.

use crate::features::db::dbpg002::commands::dbpg002_executor::Dbpg002CommandExecutor;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::Dbpg002SummaryDto;

impl Dbpg002CommandExecutor {
    pub(super) async fn get_summary(&self) -> Result<Dbpg002SummaryDto, Dbpg002Error> {
        self.queries.get_summary().await
    }
}
