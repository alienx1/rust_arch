//! Login gate for the identity provider

use tracing::info;

use crate::features::su::supg005::commands::Supg005CommandExecutor;
use crate::features::su::supg005::errors::Supg005Error;

impl Supg005CommandExecutor {
    pub(super) async fn check_login_allowed(&self, identifier: &str) -> Result<bool, Supg005Error> {
        let allowed = self.queries.is_login_allowed(identifier).await?;
        if !allowed {
            info!("Login refused for inactive account: {identifier}");
        }
        Ok(allowed)
    }
}
