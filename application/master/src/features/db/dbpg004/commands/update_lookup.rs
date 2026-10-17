//! DBPG004 command: update lookup labels and active flag with optimistic lock verification

use sea_orm::{EntityTrait, Set, TransactionTrait};
use uuid::Uuid;

use crate::features::optimistic_update::update_versioned;

use crate::features::db::dbpg004::errors::Dbpg004Error;
use crate::features::db::dbpg004::models::{
    ENGLISH_LANGUAGE_ID, LookupDto, THAI_LANGUAGE_ID, UpdateLookupCommand, db_master_lookup,
    is_protected_category,
};

use super::dbpg004_executor::Dbpg004CommandExecutor;
use super::lookup_label_upsert::upsert_label;
use super::lookup_rules::required_label;

impl Dbpg004CommandExecutor {
    pub(super) async fn update_lookup(
        &self,
        id: Uuid,
        cmd: UpdateLookupCommand,
        actor_id: Option<Uuid>,
    ) -> Result<LookupDto, Dbpg004Error> {
        let name_th = required_label(&cmd.name_th)?;

        let existing = db_master_lookup::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg004Error::NotFound(id.to_string()))?;
        if is_protected_category(&existing.category) && existing.active && !cmd.active {
            return Err(Dbpg004Error::Validation(format!(
                "'{}' is a system-protected value and cannot be deactivated",
                existing.code
            )));
        }

        let next_version = existing.row_version + 1;
        let txn = self.db.begin().await?;
        let mut am: db_master_lookup::ActiveModel = existing.into();
        am.active = Set(cmd.active);
        am.change_by = Set(actor_id);
        update_versioned(&txn, am, (db_master_lookup::Column::Id, db_master_lookup::Column::RowVersion), id, cmd.row_version).await?;

        upsert_label(&txn, id, THAI_LANGUAGE_ID, name_th, actor_id, next_version).await?;
        let name_en = cmd.name_en.trim().to_string();
        if !name_en.is_empty() {
            upsert_label(
                &txn,
                id,
                ENGLISH_LANGUAGE_ID,
                name_en,
                actor_id,
                next_version,
            )
            .await?;
        }
        txn.commit().await?;

        self.queries
            .get_lookup_by_id(id)
            .await?
            .ok_or_else(|| Dbpg004Error::Internal("Failed to retrieve updated lookup".to_string()))
    }
}
