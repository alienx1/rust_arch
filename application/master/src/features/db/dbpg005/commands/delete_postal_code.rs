//! Delete a postal code for real: it owns no rows, so anything referencing it refuses the delete

use sea_orm::{EntityTrait, TransactionTrait};
use uuid::Uuid;

use super::dbpg005_executor::Dbpg005CommandExecutor;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::db_postal_code;
use crate::features::referential_integrity::is_foreign_key_violation;

impl Dbpg005CommandExecutor {
    pub(super) async fn delete_postal_code(&self, id: Uuid) -> Result<(), Dbpg005Error> {
        db_postal_code::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg005Error::NotFound(id.to_string()))?;

        let txn = self.db.begin().await?;
        match db_postal_code::Entity::delete_by_id(id).exec(&txn).await {
            Ok(_) => {
                txn.commit().await?;
                Ok(())
            }
            Err(err) if is_foreign_key_violation(&err) => {
                let _ = txn.rollback().await;
                Err(Dbpg005Error::InUse(
                    "This postal code is still referenced; switch it off instead of deleting it".to_string(),
                ))
            }
            Err(err) => {
                let _ = txn.rollback().await;
                Err(err.into())
            }
        }
    }
}
