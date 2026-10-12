//! SUPG003 delete-group action: removes a group together with its matrix, but only when no
//! user (primary group or scope assignment) holds it.

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use uuid::Uuid;

use super::supg003_executor::Supg003CommandExecutor;
use crate::features::referential_integrity::is_foreign_key_violation;
use crate::features::su::supg003::errors::Supg003Error;
use crate::features::su::supg003::models::{
    is_admin_group, su_permission, su_permission_group,
};
use crate::features::su::supg003::queries::group_user_ids;

impl Supg003CommandExecutor {
    pub(super) async fn delete_group(&self, id: Uuid) -> Result<(), Supg003Error> {
        let existing = su_permission_group::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg003Error::NotFound(id.to_string()))?;
        if is_admin_group(&existing.code) {
            return Err(Supg003Error::Validation("The ADMIN group cannot be deleted".to_string()));
        }
        // Primary-group users and users holding it through a scope assignment
        let users = group_user_ids(&self.db, id).await?.len();
        if users > 0 {
            return Err(Supg003Error::InUse(format!(
                "The group still has {users} user(s): move them to another group first"
            )));
        }

        // Owned data first (the matrix), then the group, in one transaction
        let txn = self.db.begin().await?;
        let outcome = async {
            su_permission::Entity::delete_many()
                .filter(su_permission::Column::PermissionGroupId.eq(id))
                .exec(&txn)
                .await?;
            su_permission_group::Entity::delete_by_id(id).exec(&txn).await
        }
        .await;
        match outcome {
            Ok(_) => {
                txn.commit().await?;
                Ok(())
            }
            Err(e) if is_foreign_key_violation(&e) => {
                let _ = txn.rollback().await;
                Err(Supg003Error::InUse(
                    "The group is still referenced by other records: switch it off instead".to_string(),
                ))
            }
            Err(e) => {
                let _ = txn.rollback().await;
                Err(e.into())
            }
        }
    }
}
