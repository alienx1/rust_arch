//! DBPG006 command: validate and create a label entry (all its languages in one transaction)

use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, TransactionTrait};
use tracing::info;
use uuid::Uuid;

use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::{CreateLabelCommand, LabelEntryDto, db_label};

use super::dbpg006_executor::Dbpg006CommandExecutor;
use super::label_rules::{normalize_label_category, normalize_label_key, normalize_values};
use super::label_value_writer::{ensure_active_languages, write_values};

impl Dbpg006CommandExecutor {
    pub(super) async fn create_label(
        &self,
        cmd: CreateLabelCommand,
        actor_id: Option<Uuid>,
    ) -> Result<LabelEntryDto, Dbpg006Error> {
        let category = normalize_label_category(&cmd.category)?;
        let key = normalize_label_key(&cmd.key)?;
        let values = normalize_values(&cmd.values)?;
        ensure_active_languages(&self.db, &values).await?;

        let exists = db_label::Entity::find()
            .filter(db_label::Column::Category.eq(category.as_str()))
            .filter(db_label::Column::Key.eq(key.as_str()))
            .count(&self.db)
            .await?
            > 0;
        if exists {
            return Err(Dbpg006Error::Duplicate(format!(
                "Label '{key}' already exists in category '{category}'"
            )));
        }

        info!("Creating label: category={category}, key={key}");
        let txn = self.db.begin().await?;
        write_values(&txn, &category, &key, values, actor_id).await?;
        txn.commit().await?;

        self.queries
            .get_label(&category, &key)
            .await?
            .ok_or_else(|| Dbpg006Error::Internal("Failed to retrieve created label".to_string()))
    }
}
