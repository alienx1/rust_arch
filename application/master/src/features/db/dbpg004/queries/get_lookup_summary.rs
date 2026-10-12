//! DBPG004 query: retrieve the lookup KPI summary

use crate::features::db::dbpg004::errors::Dbpg004Error;
use crate::features::db::dbpg004::models::Dbpg004SummaryDto;

use super::db_dbpg004_queries::DbDbpg004Queries;

impl DbDbpg004Queries {
    pub(super) async fn get_summary(&self) -> Result<Dbpg004SummaryDto, Dbpg004Error> {
        let categories = self.list_categories().await?;
        Ok(Dbpg004SummaryDto {
            total_lookups: categories.iter().map(|c| c.total).sum(),
            active_lookups: categories.iter().map(|c| c.active).sum(),
            total_categories: categories.len() as u64,
        })
    }
}
