//! SUPG004 delete-user action: removes the local account, then its sign-in identity at the
//! identity provider (best effort: the login guard already refuses accounts without a row).

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use tracing::warn;
use uuid::Uuid;

use super::supg004_executor::Supg004CommandExecutor;
use crate::features::referential_integrity::is_foreign_key_violation;
use crate::features::su::supg004::errors::Supg004Error;
use crate::features::su::supg004::models::su_user;
use domain::entity::su_user_assignment;

impl Supg004CommandExecutor {
    pub(super) async fn delete_user(&self, id: Uuid, actor_id: Option<Uuid>) -> Result<(), Supg004Error> {
        if actor_id == Some(id) {
            return Err(Supg004Error::NotAllowed("You cannot delete your own account".to_string()));
        }
        let existing = su_user::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg004Error::NotFound(id.to_string()))?;

        // Assignments first (the database cascades too), both in one transaction
        let txn = self.db.begin().await?;
        let deleted = async {
            su_user_assignment::Entity::delete_many()
                .filter(su_user_assignment::Column::UserId.eq(id))
                .exec(&txn)
                .await?;
            su_user::Entity::delete_by_id(id).exec(&txn).await?;
            Ok::<(), sea_orm::DbErr>(())
        }
        .await;
        match deleted {
            Ok(()) => txn.commit().await?,
            Err(e) if is_foreign_key_violation(&e) => {
                return Err(Supg004Error::InUse(
                    "The account is still referenced by other records: switch it off instead".to_string(),
                ));
            }
            Err(e) => return Err(e.into()),
        }

        // The row is gone: sessions are looked up by login name, then the identity is removed
        let username = existing.username;
        if let Err(e) = self.sessions.revoke_user_sessions(&username).await {
            warn!("Could not revoke the sessions of deleted account {username}: {e}");
        }
        let Some(identity) = &self.identity else { return Ok(()) };
        match identity.find_identity_id(&username).await {
            Ok(Some(identity_id)) => {
                if let Err(e) = identity.delete_identity(&identity_id).await {
                    warn!("Account {username} deleted but its identity {identity_id} could not be removed: {e}");
                }
            }
            Ok(None) => {}
            Err(e) => warn!("Account {username} deleted but its identity could not be looked up: {e}"),
        }
        Ok(())
    }
}
