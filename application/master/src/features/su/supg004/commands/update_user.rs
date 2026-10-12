//! SUPG004 update-user action: replaces an account's assignments and/or switches it on / off
//! (identity provider first, then the local flag).

use sea_orm::{ActiveModelTrait, EntityTrait, Set, TransactionTrait};
use std::collections::HashSet;
use uuid::Uuid;

use super::supg004_executor::Supg004CommandExecutor;
use crate::features::su::supg004::errors::Supg004Error;
use crate::features::su::supg004::models::{UpdateUserCommand, UserAccountDto, is_admin_group, su_user};

impl Supg004CommandExecutor {
    pub(super) async fn update_user(
        &self,
        id: Uuid,
        cmd: UpdateUserCommand,
        actor_id: Option<Uuid>,
        can_assign_admin: bool,
    ) -> Result<UserAccountDto, Supg004Error> {
        let existing = su_user::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg004Error::NotFound(id.to_string()))?;
        if existing.row_version != cmd.row_version {
            return Err(Supg004Error::VersionConflict);
        }

        let (assignments, groups) = self.validate_assignments(&cmd.assignments).await?;
        let primary_group = assignments[0].0;

        let old = self.assignment_rows(id).await?;
        let old_keys: HashSet<(Uuid, Uuid)> = old.iter().map(|(_, (g, _, u))| (*g, *u)).collect();
        let new_keys: HashSet<(Uuid, Uuid)> = assignments.iter().map(|(g, _, u)| (*g, *u)).collect();
        let assignments_changed = old_keys != new_keys || primary_group != existing.permission_group_id;
        let active_changed = cmd.active != existing.active;
        if actor_id == Some(id) && (assignments_changed || active_changed) {
            return Err(Supg004Error::NotAllowed(
                "You cannot change your own assignments or switch off your own account".to_string(),
            ));
        }

        if assignments_changed {
            // Moving an account into or out of the ADMIN group needs APPROVE
            let admin_groups: HashSet<Uuid> =
                groups.iter().filter(|(_, code)| is_admin_group(code)).map(|(g, _)| *g).collect();
            let old_is_admin = self
                .group(existing.permission_group_id)
                .await
                .map(|g| is_admin_group(&g.code))
                .unwrap_or(false);
            let touches_admin = new_keys.difference(&old_keys).any(|(g, _)| admin_groups.contains(g))
                || old_keys.difference(&new_keys).any(|(g, _)| admin_groups.contains(g))
                || (old_is_admin && admin_groups.is_empty());
            // Removed rows of a group that is not part of the new picks may still be ADMIN
            let removed_admin = {
                let mut found = false;
                for (g, _) in old_keys.difference(&new_keys) {
                    if !groups.contains_key(g) && self.group(*g).await.map(|m| is_admin_group(&m.code)).unwrap_or(false) {
                        found = true;
                    }
                }
                found
            };
            if (touches_admin || removed_admin) && !can_assign_admin {
                return Err(Supg004Error::NotAllowed(
                    "Moving an account into or out of the ADMIN group needs APPROVE on SUPG004".to_string(),
                ));
            }

            let next = existing.row_version + 1;
            let txn = self.db.begin().await?;
            let mut am: su_user::ActiveModel = existing.into();
            am.permission_group_id = Set(primary_group);
            am.change_by = Set(actor_id);
            am.row_version = Set(next);
            am.update(&txn).await?;
            self.write_assignments(&txn, id, &assignments, actor_id).await?;
            txn.commit().await?;
        }

        // Identity provider first (inside), then the local flag
        if active_changed {
            let user_ref = id.to_string();
            let outcome = if cmd.active {
                self.sessions.reactivate_user(&user_ref).await
            } else {
                self.sessions.suspend_user(&user_ref).await
            };
            outcome.map_err(|e| Supg004Error::IdentityProvider(e.to_string()))?;
        }

        self.queries
            .get_user(id)
            .await?
            .ok_or_else(|| Supg004Error::Internal("Failed to retrieve updated account".to_string()))
    }
}
