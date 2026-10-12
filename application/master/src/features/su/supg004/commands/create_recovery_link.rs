//! SUPG004 create-recovery-link action: issues a one-time password link for an active account.

use sea_orm::EntityTrait;
use uuid::Uuid;

use super::identity_rules::provider_error;
use super::supg004_executor::Supg004CommandExecutor;
use crate::features::su::supg004::errors::Supg004Error;
use crate::features::su::supg004::models::{RecoveryLinkDto, su_user};

impl Supg004CommandExecutor {
    pub(super) async fn create_recovery_link(&self, id: Uuid) -> Result<RecoveryLinkDto, Supg004Error> {
        let identity = self.identity()?;
        let user = su_user::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg004Error::NotFound(id.to_string()))?;
        if !user.active {
            return Err(Supg004Error::Validation(
                "The account is switched off: switch it on before issuing a recovery link".to_string(),
            ));
        }
        let link = identity.create_recovery_link(&user.username).await.map_err(provider_error)?;
        Ok(RecoveryLinkDto { url: link.url, expires_at: link.expires_at })
    }
}
