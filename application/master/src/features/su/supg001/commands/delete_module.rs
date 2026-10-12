//! SUPG001 delete a module: its labels go with it, anything else still referencing it blocks

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use uuid::Uuid;

use super::supg001_executor::Supg001CommandExecutor;
use super::unit_rules::in_use_or_database;
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{db_module, db_module_label, is_protected_module};

impl Supg001CommandExecutor {
    pub(super) async fn delete_module(&self, id: Uuid) -> Result<(), Supg001Error> {
        let existing = db_module::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg001Error::NotFound(id.to_string()))?;
        if is_protected_module(&existing.code) {
            return Err(Supg001Error::Validation(format!(
                "Module '{}' hosts the security administration and cannot be deleted",
                existing.code
            )));
        }

        let txn = self.db.begin().await?;
        db_module_label::Entity::delete_many()
            .filter(db_module_label::Column::ModuleId.eq(id))
            .exec(&txn)
            .await?;
        db_module::Entity::delete_by_id(id).exec(&txn).await.map_err(in_use_or_database)?;
        txn.commit().await?;
        Ok(())
    }
}
