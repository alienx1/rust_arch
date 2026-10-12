//! SUPG001 create a system: unique code, unit + Thai/English labels in one transaction

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tracing::info;
use uuid::Uuid;

use super::supg001_executor::Supg001CommandExecutor;
use super::unit_label_writes::put_system_label;
use super::unit_rules::{english_or, normalize_unit_code, thai_name};
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{
    CreateSystemCommand, ENGLISH_LANGUAGE_ID, SystemDto, THAI_LANGUAGE_ID, db_system,
};

impl Supg001CommandExecutor {
    pub(super) async fn create_system(
        &self,
        cmd: CreateSystemCommand,
        actor_id: Option<Uuid>,
    ) -> Result<SystemDto, Supg001Error> {
        let code = normalize_unit_code(&cmd.code)?;
        let th = thai_name(&cmd.name_th)?;
        let en = english_or(&cmd.name_en, &th);

        if db_system::Entity::find()
            .filter(db_system::Column::Code.eq(&code))
            .one(&self.db)
            .await?
            .is_some()
        {
            return Err(Supg001Error::DuplicateCode(format!("System code '{code}' already exists")));
        }

        let id = Uuid::now_v7();
        info!("Creating new system: id={id}, code={code}");
        let txn = self.db.begin().await?;
        db_system::ActiveModel {
            id: Set(id),
            code: Set(code),
            active: Set(cmd.active),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
        .insert(&txn)
        .await?;
        put_system_label(&txn, id, THAI_LANGUAGE_ID, th, actor_id, 1).await?;
        put_system_label(&txn, id, ENGLISH_LANGUAGE_ID, en, actor_id, 1).await?;
        txn.commit().await?;

        self.queries
            .get_system(id)
            .await?
            .ok_or_else(|| Supg001Error::Internal("Failed to retrieve created system".to_string()))
    }
}
