//! DBPG006 query: one entry with all its translations

use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::LabelEntryDto;

use super::db_dbpg006_queries::DbDbpg006Queries;

impl DbDbpg006Queries {
    pub(super) async fn get_label(
        &self,
        category: &str,
        key: &str,
    ) -> Result<Option<LabelEntryDto>, Dbpg006Error> {
        let pair = (category.trim().to_string(), key.trim().to_string());
        Ok(self.entries_for(&[pair]).await?.into_iter().next())
    }
}
