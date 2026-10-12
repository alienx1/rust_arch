//! Revoke a single device session

use tracing::info;

use crate::features::su::supg005::commands::Supg005CommandExecutor;
use crate::features::su::supg005::errors::Supg005Error;

impl Supg005CommandExecutor {
    pub(super) async fn revoke_session(&self, session_id: &str) -> Result<bool, Supg005Error> {
        info!("Revoking single device session: {session_id}");
        if let Some(ref rev) = self.revocation {
            return rev.revoke_session(session_id).await.map_err(Supg005Error::Kratos);
        }
        Ok(true)
    }
}
