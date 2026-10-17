//! DBPG003 write: hard-delete an employee together with the person name it owns

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use uuid::Uuid;

use crate::features::db::dbpg003::commands::dbpg003_executor::Dbpg003CommandExecutor;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::{db_employee, db_person_name, db_person_name_label};
use crate::features::db::contacts::delete_contact_group;
use crate::features::referential_integrity::is_foreign_key_violation;

impl Dbpg003CommandExecutor {
    /// Deletes the person name labels and person name (owned by the employee), then the employee.
    /// A foreign-key violation (a `su_user` account) rolls everything back and reports `InUse`.
    pub(super) async fn delete_employee(&self, id: Uuid) -> Result<(), Dbpg003Error> {
        let existing = db_employee::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg003Error::NotFound(id.to_string()))?;
        let person_id = existing.person_name_id;
        let image_key = existing.image_key.clone();
        let contact_group = existing.contact_group_id;

        let txn = self.db.begin().await?;
        let result = async {
            db_employee::Entity::delete_by_id(id).exec(&txn).await?;
            db_person_name_label::Entity::delete_many()
                .filter(db_person_name_label::Column::PersonNameId.eq(person_id))
                .exec(&txn)
                .await?;
            db_person_name::Entity::delete_by_id(person_id)
                .exec(&txn)
                .await?;
            if let Some(group) = contact_group {
                delete_contact_group(&txn, group).await?;
            }
            Ok::<(), sea_orm::DbErr>(())
        }
        .await;

        match result {
            Ok(()) => {
                txn.commit().await?;
                if let Some(key) = image_key {
                    self.forget_object(&key).await;
                }
                Ok(())
            }
            Err(err) => {
                let _ = txn.rollback().await;
                if is_foreign_key_violation(&err) {
                    Err(Dbpg003Error::InUse(
                        "This employee is still referenced (e.g. by a user account: remove the account in SUPG004 first); switch it off instead of deleting it"
                            .to_string(),
                    ))
                } else {
                    Err(err.into())
                }
            }
        }
    }
}
