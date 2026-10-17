//! SUPG003 update-group action: switches a group on / off under an optimistic lock.

use sea_orm::{TransactionTrait, EntityTrait, Set};
use uuid::Uuid;

use crate::features::optimistic_update::update_versioned;

use super::supg003_executor::Supg003CommandExecutor;
use crate::features::su::supg003::errors::Supg003Error;
use crate::features::su::supg003::models::{
    PermissionGroupDto, UpdatePermissionGroupCommand, is_admin_group, su_permission_group,
};
use crate::features::su::supg003::queries::group_user_ids;

impl Supg003CommandExecutor {
    pub(super) async fn update_group(
        &self,
        id: Uuid,
        cmd: UpdatePermissionGroupCommand,
        actor_id: Option<Uuid>,
    ) -> Result<PermissionGroupDto, Supg003Error> {
        let existing = su_permission_group::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg003Error::NotFound(id.to_string()))?;
        if existing.active && !cmd.active {
            if is_admin_group(&existing.code) {
                return Err(Supg003Error::Validation("The ADMIN group cannot be switched off".to_string()));
            }
            let users = group_user_ids(&self.db, id).await?.len();
            if users > 0 {
                return Err(Supg003Error::Validation(format!(
                    "The group still has {users} user(s): move them to another group first"
                )));
            }
        }

        let _next = existing.row_version + 1;
        let txn = self.db.begin().await?;
        let mut am: su_permission_group::ActiveModel = existing.into();
        am.active = Set(cmd.active);
        am.change_by = Set(actor_id);
        update_versioned(&txn, am, (su_permission_group::Column::Id, su_permission_group::Column::RowVersion), id, cmd.row_version).await?;
        txn.commit().await?;

        self.queries
            .get_group(id)
            .await?
            .ok_or_else(|| Supg003Error::Internal("Failed to retrieve updated group".to_string()))
    }
}
