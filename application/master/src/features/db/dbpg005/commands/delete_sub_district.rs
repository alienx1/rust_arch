//! Delete a sub-district for real: its label rows go with it; anything still referencing it refuses the delete

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use uuid::Uuid;

use super::dbpg005_executor::Dbpg005CommandExecutor;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{db_sub_district, db_sub_district_label};
use crate::features::referential_integrity::is_foreign_key_violation;

impl Dbpg005CommandExecutor {
    pub(super) async fn delete_sub_district(&self, id: Uuid) -> Result<(), Dbpg005Error> {
        db_sub_district::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg005Error::NotFound(id.to_string()))?;

        let txn = self.db.begin().await?;
        let outcome = async {
            db_sub_district_label::Entity::delete_many()
                .filter(db_sub_district_label::Column::SubDistrictId.eq(id))
                .exec(&txn)
                .await?;
            db_sub_district::Entity::delete_by_id(id).exec(&txn).await?;
            Ok::<(), sea_orm::DbErr>(())
        }
        .await;
        match outcome {
            Ok(()) => {
                txn.commit().await?;
                Ok(())
            }
            Err(err) if is_foreign_key_violation(&err) => {
                let _ = txn.rollback().await;
                Err(Dbpg005Error::InUse(
                    "This sub-district is still referenced; switch it off instead of deleting it".to_string(),
                ))
            }
            Err(err) => {
                let _ = txn.rollback().await;
                Err(err.into())
            }
        }
    }
}
