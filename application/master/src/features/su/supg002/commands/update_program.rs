//! SUPG002 update a program: optimistic lock, protected programs stay on, no move below itself

use sea_orm::{ActiveModelTrait, EntityTrait, Set, TransactionTrait};
use uuid::Uuid;

use super::program_label_writes::put_label;
use super::program_rules::{normalize_icon, normalize_program_path, thai_name};
use super::supg002_executor::Supg002CommandExecutor;
use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::{
    ENGLISH_LANGUAGE_ID, ProgramDto, THAI_LANGUAGE_ID, UpdateProgramCommand, db_program,
    is_protected_program,
};

impl Supg002CommandExecutor {
    pub(super) async fn update_program(
        &self,
        id: Uuid,
        cmd: UpdateProgramCommand,
        actor_id: Option<Uuid>,
    ) -> Result<ProgramDto, Supg002Error> {
        let path = normalize_program_path(&cmd.path)?;
        let icon = normalize_icon(&cmd.icon)?;
        let th = thai_name(&cmd.name_th)?;

        let existing = db_program::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg002Error::NotFound(id.to_string()))?;
        if existing.row_version != cmd.row_version {
            return Err(Supg002Error::VersionConflict);
        }
        if existing.active && !cmd.active && is_protected_program(&existing.code) {
            return Err(Supg002Error::Validation(format!(
                "'{}' administers the menu and permissions and cannot be switched off",
                existing.code
            )));
        }
        self.check_parent(cmd.parent_id, existing.system_id, Some(id)).await?;

        let next = existing.row_version + 1;
        let txn = self.db.begin().await?;
        let mut am: db_program::ActiveModel = existing.into();
        am.program_id = Set(cmd.parent_id);
        am.path = Set(path);
        am.icon = Set(icon);
        am.sort_order = Set(cmd.sort_order);
        am.active = Set(cmd.active);
        am.change_by = Set(actor_id);
        am.row_version = Set(next);
        am.update(&txn).await?;
        put_label(&txn, id, THAI_LANGUAGE_ID, th, actor_id, next).await?;
        if !cmd.name_en.trim().is_empty() {
            put_label(&txn, id, ENGLISH_LANGUAGE_ID, cmd.name_en.trim().to_string(), actor_id, next).await?;
        }
        txn.commit().await?;

        self.queries
            .get_program(id)
            .await?
            .ok_or_else(|| Supg002Error::Internal("Failed to retrieve updated program".to_string()))
    }
}
