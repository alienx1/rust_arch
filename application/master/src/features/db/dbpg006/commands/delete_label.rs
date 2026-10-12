//! DBPG006 command: hard-delete every row of a label entry (labels are leaf rows)

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::db_label;

use super::dbpg006_executor::Dbpg006CommandExecutor;

impl Dbpg006CommandExecutor {
    pub(super) async fn delete_label(&self, category: &str, key: &str) -> Result<(), Dbpg006Error> {
        let (category, key) = (category.trim(), key.trim());
        let result = db_label::Entity::delete_many()
            .filter(db_label::Column::Category.eq(category))
            .filter(db_label::Column::Key.eq(key))
            .exec(&self.db)
            .await?;
        if result.rows_affected == 0 {
            return Err(Dbpg006Error::NotFound(format!(
                "Label '{key}' not found in category '{category}'"
            )));
        }
        Ok(())
    }
}
