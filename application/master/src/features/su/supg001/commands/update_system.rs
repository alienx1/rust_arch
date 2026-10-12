//! SUPG001 update a system: names / status with optimistic lock, protected systems stay on

use sea_orm::{ActiveModelTrait, EntityTrait, Set, TransactionTrait};
use uuid::Uuid;

use super::supg001_executor::Supg001CommandExecutor;
use super::unit_label_writes::put_system_label;
use super::unit_rules::thai_name;
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{
    ENGLISH_LANGUAGE_ID, SystemDto, THAI_LANGUAGE_ID, UpdateUnitCommand, db_system,
    is_protected_system,
};

impl Supg001CommandExecutor {
    pub(super) async fn update_system(
        &self,
        id: Uuid,
        cmd: UpdateUnitCommand,
        actor_id: Option<Uuid>,
    ) -> Result<SystemDto, Supg001Error> {
        let th = thai_name(&cmd.name_th)?;
        let existing = db_system::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg001Error::NotFound(id.to_string()))?;
        if existing.row_version != cmd.row_version {
            return Err(Supg001Error::VersionConflict);
        }
        if existing.active && !cmd.active && is_protected_system(&existing.code) {
            return Err(Supg001Error::Validation(format!(
                "System '{}' hosts the security administration and cannot be switched off",
                existing.code
            )));
        }
        let next = existing.row_version + 1;

        let txn = self.db.begin().await?;
        let mut am: db_system::ActiveModel = existing.into();
        am.active = Set(cmd.active);
        am.change_by = Set(actor_id);
        am.row_version = Set(next);
        am.update(&txn).await?;
        put_system_label(&txn, id, THAI_LANGUAGE_ID, th, actor_id, next).await?;
        if !cmd.name_en.trim().is_empty() {
            put_system_label(&txn, id, ENGLISH_LANGUAGE_ID, cmd.name_en.trim().to_string(), actor_id, next).await?;
        }
        txn.commit().await?;

        self.queries
            .get_system(id)
            .await?
            .ok_or_else(|| Supg001Error::Internal("Failed to retrieve updated system".to_string()))
    }
}
