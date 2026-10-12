//! DBPG004 command: hard-delete a lookup value together with its labels

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use uuid::Uuid;

use crate::features::db::dbpg004::errors::Dbpg004Error;
use crate::features::db::dbpg004::models::{
    db_master_lookup, db_master_lookup_label, is_protected_category,
};
use crate::features::referential_integrity::is_foreign_key_violation;

use super::dbpg004_executor::Dbpg004CommandExecutor;

impl Dbpg004CommandExecutor {
    /// Deletes the labels (owned), then the lookup. A foreign-key violation rolls everything back
    /// and reports `InUse`.
    pub(super) async fn delete_lookup(&self, id: Uuid) -> Result<(), Dbpg004Error> {
        let existing = db_master_lookup::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg004Error::NotFound(id.to_string()))?;
        if is_protected_category(&existing.category) {
            return Err(Dbpg004Error::Validation(format!(
                "'{}' is a system-protected value and cannot be deleted",
                existing.code
            )));
        }

        let txn = self.db.begin().await?;
        let result = async {
            db_master_lookup_label::Entity::delete_many()
                .filter(db_master_lookup_label::Column::MasterLookupId.eq(id))
                .exec(&txn)
                .await?;
            db_master_lookup::Entity::delete_by_id(id).exec(&txn).await?;
            Ok::<(), sea_orm::DbErr>(())
        }
        .await;

        match result {
            Ok(()) => {
                txn.commit().await?;
                Ok(())
            }
            Err(err) => {
                let _ = txn.rollback().await;
                if is_foreign_key_violation(&err) {
                    Err(Dbpg004Error::InUse(
                        "This value is still referenced by other records; switch it off instead of deleting it"
                            .to_string(),
                    ))
                } else {
                    Err(err.into())
                }
            }
        }
    }
}
