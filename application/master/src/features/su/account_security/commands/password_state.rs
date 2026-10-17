//! `su_user.must_change_password`: read at sign-in, cleared when a new password is set

use super::account_security_executor::AccountSecurityExecutor;
use crate::features::su::account_security::errors::AccountSecurityError;

impl AccountSecurityExecutor {
    pub(super) async fn change_required(&self, identifier: &str) -> Result<bool, AccountSecurityError> {
        Ok(self
            .queries
            .find_account_by_identifier(identifier)
            .await?
            .is_some_and(|a| a.must_change_password))
    }

    pub(super) async fn mark_changed(&self, identifier: &str) -> Result<(), AccountSecurityError> {
        let Some(account) = self.queries.find_account_by_identifier(identifier).await? else {
            return Ok(());
        };
        if account.must_change_password {
            self.queries.set_must_change_password(account.id, false).await?;
            tracing::info!(account = %account.username, "password changed: the account may sign in");
        }
        // A new password also ends any lock the failures put on it
        self.clear_failures(identifier).await
    }
}
