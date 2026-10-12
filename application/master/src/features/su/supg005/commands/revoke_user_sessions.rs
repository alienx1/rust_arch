//! Revoke all active sessions of a user

use tracing::info;

use crate::features::su::supg005::commands::Supg005CommandExecutor;
use crate::features::su::supg005::errors::Supg005Error;

impl Supg005CommandExecutor {
    pub(super) async fn revoke_all_sessions(&self, user_id: &str) -> Result<bool, Supg005Error> {
        info!("Revoking all active sessions for user: {user_id}");
        if let Some(ref rev) = self.revocation {
            // Kratos identity ids differ from local user ids; the login email is the join key.
            let identifier = self
                .queries
                .find_user_login_identifier(user_id)
                .await?
                .unwrap_or_else(|| user_id.to_string());
            return rev.revoke_user_sessions(&identifier).await.map_err(Supg005Error::Kratos);
        }
        Ok(true)
    }
}
