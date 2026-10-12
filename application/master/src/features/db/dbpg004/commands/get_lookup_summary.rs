//! DBPG004 command: retrieve the lookup KPI summary

use crate::features::db::dbpg004::errors::Dbpg004Error;
use crate::features::db::dbpg004::models::Dbpg004SummaryDto;

use super::dbpg004_executor::Dbpg004CommandExecutor;

impl Dbpg004CommandExecutor {
    pub(super) async fn get_summary(&self) -> Result<Dbpg004SummaryDto, Dbpg004Error> {
        self.queries.get_summary().await
    }
}
