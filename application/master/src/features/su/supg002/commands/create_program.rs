//! SUPG002 create a program: unique `(system, module, code)`, program + labels in one transaction

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tracing::info;
use uuid::Uuid;

use super::program_label_writes::put_label;
use super::program_rules::{
    english_or, normalize_icon, normalize_program_code, normalize_program_path, thai_name,
};
use super::supg002_executor::Supg002CommandExecutor;
use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::{
    CreateProgramCommand, ENGLISH_LANGUAGE_ID, ProgramDto, THAI_LANGUAGE_ID, db_program,
};
use domain::entity::{db_module, db_system};

impl Supg002CommandExecutor {
    pub(super) async fn create_program(
        &self,
        cmd: CreateProgramCommand,
        actor_id: Option<Uuid>,
    ) -> Result<ProgramDto, Supg002Error> {
        let code = normalize_program_code(&cmd.code)?;
        let path = normalize_program_path(&cmd.path)?;
        let icon = normalize_icon(&cmd.icon)?;
        let th = thai_name(&cmd.name_th)?;
        let en = english_or(&cmd.name_en, &th);

        db_system::Entity::find_by_id(cmd.system_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg002Error::NotFound(format!("System '{}' does not exist", cmd.system_id)))?;
        let module = db_module::Entity::find_by_id(cmd.module_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg002Error::NotFound(format!("Module '{}' does not exist", cmd.module_id)))?;
        if module.system_id != cmd.system_id {
            return Err(Supg002Error::Validation("The module does not belong to the system".to_string()));
        }
        self.check_parent(cmd.parent_id, cmd.system_id, None).await?;

        if db_program::Entity::find()
            .filter(db_program::Column::SystemId.eq(cmd.system_id))
            .filter(db_program::Column::ModuleId.eq(cmd.module_id))
            .filter(db_program::Column::Code.eq(&code))
            .one(&self.db)
            .await?
            .is_some()
        {
            return Err(Supg002Error::DuplicateCode(format!(
                "Program code '{code}' already exists in this module"
            )));
        }

        let id = Uuid::now_v7();
        info!("Creating new program: id={id}, code={code}");
        let txn = self.db.begin().await?;
        db_program::ActiveModel {
            id: Set(id),
            system_id: Set(cmd.system_id),
            module_id: Set(cmd.module_id),
            program_id: Set(cmd.parent_id),
            code: Set(code),
            path: Set(path),
            icon: Set(icon),
            sort_order: Set(cmd.sort_order),
            active: Set(cmd.active),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
        .insert(&txn)
        .await?;
        put_label(&txn, id, THAI_LANGUAGE_ID, th, actor_id, 1).await?;
        put_label(&txn, id, ENGLISH_LANGUAGE_ID, en, actor_id, 1).await?;
        txn.commit().await?;

        self.queries
            .get_program(id)
            .await?
            .ok_or_else(|| Supg002Error::Internal("Failed to retrieve created program".to_string()))
    }
}
