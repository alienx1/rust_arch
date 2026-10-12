//! DBPG006 command: get one entry

use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::LabelEntryDto;

use super::dbpg006_executor::Dbpg006CommandExecutor;

impl Dbpg006CommandExecutor {
    pub(super) async fn get_label(
        &self,
        category: &str,
        key: &str,
    ) -> Result<Option<LabelEntryDto>, Dbpg006Error> {
        self.queries.get_label(category, key).await
    }
}
