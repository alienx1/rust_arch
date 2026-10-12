//! SUPG003 update-group action: switches a group on / off under an optimistic lock.

use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use uuid::Uuid;

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
        if existing.row_version != cmd.row_version {
            return Err(Supg003Error::VersionConflict);
        }
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

        let next = existing.row_version + 1;
        let mut am: su_permission_group::ActiveModel = existing.into();
        am.active = Set(cmd.active);
        am.change_by = Set(actor_id);
        am.row_version = Set(next);

        am.update(&self.db).await?;

        self.queries
            .get_group(id)
            .await?
            .ok_or_else(|| Supg003Error::Internal("Failed to retrieve updated group".to_string()))
    }
}
