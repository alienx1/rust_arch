//! DBPG006 command: replace the translations of an existing label entry in one transaction

use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, TransactionTrait};
use tracing::info;
use uuid::Uuid;

use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::{LabelEntryDto, UpdateLabelCommand, db_label};

use super::dbpg006_executor::Dbpg006CommandExecutor;
use super::label_rules::{normalize_label_category, normalize_label_key, normalize_values};
use super::label_value_writer::{ensure_active_languages, write_values};

impl Dbpg006CommandExecutor {
    pub(super) async fn update_label(
        &self,
        cmd: UpdateLabelCommand,
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
        if !exists {
            return Err(Dbpg006Error::NotFound(format!(
                "Label '{key}' not found in category '{category}'"
            )));
        }

        info!("Updating label: category={category}, key={key}");
        let txn = self.db.begin().await?;
        // Lock the entry's rows and check its version (the sum of the translations' versions): an
        // entry saved since it was loaded must not be overwritten (lost update)
        let rows = sea_orm::QuerySelect::lock_exclusive(
            db_label::Entity::find()
                .filter(db_label::Column::Category.eq(category.as_str()))
                .filter(db_label::Column::Key.eq(key.as_str())),
        )
        .all(&txn)
        .await?;
        if rows.iter().map(|r| r.row_version).sum::<i64>() != cmd.row_version {
            return Err(Dbpg006Error::VersionConflict);
        }
        write_values(&txn, &category, &key, values, actor_id).await?;
        txn.commit().await?;

        self.queries
            .get_label(&category, &key)
            .await?
            .ok_or_else(|| Dbpg006Error::Internal("Failed to retrieve updated label".to_string()))
    }
}
