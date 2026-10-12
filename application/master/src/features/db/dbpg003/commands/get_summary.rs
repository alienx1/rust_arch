//! DBPG003 read: KPI summary of employees.

use crate::features::db::dbpg003::commands::dbpg003_executor::Dbpg003CommandExecutor;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::Dbpg003SummaryDto;

impl Dbpg003CommandExecutor {
    pub(super) async fn get_summary(&self) -> Result<Dbpg003SummaryDto, Dbpg003Error> {
        self.queries.get_summary().await
    }
}
