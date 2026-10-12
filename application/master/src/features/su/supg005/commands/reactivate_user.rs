//! Reactivate a suspended user account

use crate::features::su::supg005::commands::Supg005CommandExecutor;
use crate::features::su::supg005::errors::Supg005Error;

impl Supg005CommandExecutor {
    pub(super) async fn reactivate_account(&self, user_id: &str) -> Result<bool, Supg005Error> {
        self.set_account_active(user_id, true).await
    }
}
