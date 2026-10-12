//! DBPG006 command: list categories with their entry counts

use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::LabelCategoryDto;

use super::dbpg006_executor::Dbpg006CommandExecutor;

impl Dbpg006CommandExecutor {
    pub(super) async fn list_categories(&self) -> Result<Vec<LabelCategoryDto>, Dbpg006Error> {
        self.queries.list_categories().await
    }
}
