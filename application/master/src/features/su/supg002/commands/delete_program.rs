//! SUPG002 delete a program: its labels and permission grants go with it, child programs block

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use uuid::Uuid;

use super::supg002_executor::Supg002CommandExecutor;
use crate::features::referential_integrity::is_foreign_key_violation;
use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::{
    db_program, db_program_label, is_protected_program, su_permission,
};

impl Supg002CommandExecutor {
    pub(super) async fn delete_program(&self, id: Uuid) -> Result<(), Supg002Error> {
        let existing = db_program::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg002Error::NotFound(id.to_string()))?;
        if is_protected_program(&existing.code) {
            return Err(Supg002Error::Validation(format!(
                "'{}' administers the menu and permissions and cannot be deleted",
                existing.code
            )));
        }

        let txn = self.db.begin().await?;
        db_program_label::Entity::delete_many()
            .filter(db_program_label::Column::ProgramId.eq(id))
            .exec(&txn)
            .await?;
        su_permission::Entity::delete_many()
            .filter(su_permission::Column::ProgramId.eq(id))
            .exec(&txn)
            .await?;
        if let Err(err) = db_program::Entity::delete_by_id(id).exec(&txn).await {
            if is_foreign_key_violation(&err) {
                return Err(Supg002Error::InUse(
                    "This program is still referenced by other data and cannot be deleted; switch it off instead"
                        .to_string(),
                ));
            }
            return Err(err.into());
        }
        txn.commit().await?;
        Ok(())
    }
}
