//! SUPG001 create a module: code unique within the system, unit + labels in one transaction

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tracing::info;
use uuid::Uuid;

use super::supg001_executor::Supg001CommandExecutor;
use super::unit_label_writes::put_module_label;
use super::unit_rules::{english_or, normalize_unit_code, thai_name};
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{
    CreateModuleCommand, ENGLISH_LANGUAGE_ID, ModuleDto, THAI_LANGUAGE_ID, db_module, db_system,
};

impl Supg001CommandExecutor {
    pub(super) async fn create_module(
        &self,
        cmd: CreateModuleCommand,
        actor_id: Option<Uuid>,
    ) -> Result<ModuleDto, Supg001Error> {
        let code = normalize_unit_code(&cmd.code)?;
        let th = thai_name(&cmd.name_th)?;
        let en = english_or(&cmd.name_en, &th);

        let system = db_system::Entity::find_by_id(cmd.system_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg001Error::NotFound(format!("System '{}' does not exist", cmd.system_id)))?;
        if db_module::Entity::find()
            .filter(db_module::Column::SystemId.eq(system.id))
            .filter(db_module::Column::Code.eq(&code))
            .one(&self.db)
            .await?
            .is_some()
        {
            return Err(Supg001Error::DuplicateCode(format!(
                "Module code '{code}' already exists in this system"
            )));
        }

        let id = Uuid::now_v7();
        info!("Creating new module: id={id}, code={code}");
        let txn = self.db.begin().await?;
        db_module::ActiveModel {
            id: Set(id),
            system_id: Set(system.id),
            code: Set(code),
            active: Set(cmd.active),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
        .insert(&txn)
        .await?;
        put_module_label(&txn, id, THAI_LANGUAGE_ID, th, actor_id, 1).await?;
        put_module_label(&txn, id, ENGLISH_LANGUAGE_ID, en, actor_id, 1).await?;
        txn.commit().await?;

        self.queries
            .get_module(id)
            .await?
            .ok_or_else(|| Supg001Error::Internal("Failed to retrieve created module".to_string()))
    }
}
