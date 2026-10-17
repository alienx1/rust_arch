//! Forced password change: flag the account, end its sessions, mail the link

use uuid::Uuid;

use super::account_security_executor::AccountSecurityExecutor;
use crate::features::su::account_security::errors::AccountSecurityError;
use crate::features::su::account_security::models::{PasswordMailKind, PasswordMailResult};

impl AccountSecurityExecutor {
    pub(super) async fn force_change(&self, user_id: Uuid) -> Result<PasswordMailResult, AccountSecurityError> {
        let account = self
            .queries
            .find_account(user_id)
            .await?
            .ok_or_else(|| AccountSecurityError::NotFound(user_id.to_string()))?;
        if !account.active {
            return Err(AccountSecurityError::Validation(
                "The account is switched off: switch it on before forcing a password change".to_string(),
            ));
        }
        self.queries.set_must_change_password(user_id, true).await?;
        // The change takes effect now: whoever is signed in with the old password is signed out
        if let Some(revocation) = &self.revocation
            && let Err(e) = revocation.revoke_user_sessions(&account.username).await
        {
            tracing::warn!(account = %account.username, "sessions not ended after a forced password change: {e}");
        }
        self.mail_link_of(&account.username, PasswordMailKind::Forced).await
    }
}
