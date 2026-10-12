//! DBPG004 command: list lookup categories with row counts

use crate::features::db::dbpg004::errors::Dbpg004Error;
use crate::features::db::dbpg004::models::LookupCategoryDto;

use super::dbpg004_executor::Dbpg004CommandExecutor;

impl Dbpg004CommandExecutor {
    pub(super) async fn list_categories(&self) -> Result<Vec<LookupCategoryDto>, Dbpg004Error> {
        self.queries.list_categories().await
    }
}
