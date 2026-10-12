//! DBPG000 command: retrieve KPI summary of organizations and companies

use crate::features::db::dbpg000::errors::Dbpg000Error;
use crate::features::db::dbpg000::models::OrganizationSummaryDto;

use super::dbpg000_executor::Dbpg000CommandExecutor;

impl Dbpg000CommandExecutor {
    pub(super) async fn get_summary(&self) -> Result<OrganizationSummaryDto, Dbpg000Error> {
        self.queries.get_summary().await
    }
}
