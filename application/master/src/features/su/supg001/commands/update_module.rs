//! SUPG001 update a module: names / status with optimistic lock, protected modules stay on

use sea_orm::{EntityTrait, Set, TransactionTrait};
use uuid::Uuid;

use crate::features::optimistic_update::update_versioned;

use super::supg001_executor::Supg001CommandExecutor;
use super::unit_label_writes::put_module_label;
use super::unit_rules::thai_name;
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{
    ENGLISH_LANGUAGE_ID, ModuleDto, THAI_LANGUAGE_ID, UpdateUnitCommand, db_module,
    is_protected_module,
};

impl Supg001CommandExecutor {
    pub(super) async fn update_module(
        &self,
        id: Uuid,
        cmd: UpdateUnitCommand,
        actor_id: Option<Uuid>,
    ) -> Result<ModuleDto, Supg001Error> {
        let th = thai_name(&cmd.name_th)?;
        let existing = db_module::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg001Error::NotFound(id.to_string()))?;
        if existing.active && !cmd.active && is_protected_module(&existing.code) {
            return Err(Supg001Error::Validation(format!(
                "Module '{}' hosts the security administration and cannot be switched off",
                existing.code
            )));
        }
        let next = existing.row_version + 1;

        let txn = self.db.begin().await?;
        let mut am: db_module::ActiveModel = existing.into();
        am.active = Set(cmd.active);
        am.change_by = Set(actor_id);
        update_versioned(&txn, am, (db_module::Column::Id, db_module::Column::RowVersion), id, cmd.row_version).await?;
        put_module_label(&txn, id, THAI_LANGUAGE_ID, th, actor_id, next).await?;
        if !cmd.name_en.trim().is_empty() {
            put_module_label(&txn, id, ENGLISH_LANGUAGE_ID, cmd.name_en.trim().to_string(), actor_id, next).await?;
        }
        txn.commit().await?;

        self.queries
            .get_module(id)
            .await?
            .ok_or_else(|| Supg001Error::Internal("Failed to retrieve updated module".to_string()))
    }
}
