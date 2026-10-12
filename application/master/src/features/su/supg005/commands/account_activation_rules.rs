//! Shared account activation flow used by suspend and reactivate

use tracing::info;

use crate::features::su::supg005::commands::Supg005CommandExecutor;
use crate::features::su::supg005::errors::Supg005Error;

impl Supg005CommandExecutor {
    /// Identity provider first (security-critical), then the local flag.
    pub(super) async fn set_account_active(&self, user_id: &str, active: bool) -> Result<bool, Supg005Error> {
        info!("Setting account active={active} for user: {user_id}");
        if let Some(ref rev) = self.revocation {
            let identifier = self
                .queries
                .find_user_login_identifier(user_id)
                .await?
                .unwrap_or_else(|| user_id.to_string());
            rev.set_user_active(&identifier, active).await.map_err(Supg005Error::Kratos)?;
        }
        if !self.queries.set_user_active(user_id, active).await? {
            return Err(Supg005Error::UserNotFound(format!("User {user_id} not found")));
        }
        Ok(true)
    }
}
