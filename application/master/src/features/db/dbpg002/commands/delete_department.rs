//! DBPG002 write: hard-delete a department with its owned labels

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use uuid::Uuid;

use crate::features::db::dbpg002::commands::dbpg002_executor::Dbpg002CommandExecutor;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{db_department, db_department_label};
use crate::features::referential_integrity::is_foreign_key_violation;

impl Dbpg002CommandExecutor {
    pub(super) async fn delete_department(&self, id: Uuid) -> Result<(), Dbpg002Error> {
        db_department::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg002Error::NotFound(id.to_string()))?;

        let txn = self.db.begin().await?;
        db_department_label::Entity::delete_many()
            .filter(db_department_label::Column::DepartmentId.eq(id))
            .exec(&txn)
            .await?;
        if let Err(err) = db_department::Entity::delete_by_id(id).exec(&txn).await {
            return Err(if is_foreign_key_violation(&err) {
                Dbpg002Error::InUse(
                    "This department is still referenced by other records; switch it off instead"
                        .to_string(),
                )
            } else {
                err.into()
            });
        }
        txn.commit().await?;
        Ok(())
    }
}
