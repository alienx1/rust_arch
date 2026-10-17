//! Password mail: a one-time link, e-mailed to the account holder

use uuid::Uuid;

use super::account_security_executor::AccountSecurityExecutor;
use super::login_lock::account_key;
use crate::features::su::account_security::errors::AccountSecurityError;
use crate::features::su::account_security::models::{
    PasswordMailKind, PasswordMailResult, mask_email, password_mail_message,
};

/// How long the link lives (the identity provider is asked for this)
const LINK_VALID_HOURS: u32 = 24;

impl AccountSecurityExecutor {
    pub(super) async fn mail_password_link(
        &self,
        user_id: Uuid,
        kind: PasswordMailKind,
    ) -> Result<PasswordMailResult, AccountSecurityError> {
        let account = self
            .queries
            .find_account(user_id)
            .await?
            .ok_or_else(|| AccountSecurityError::NotFound(user_id.to_string()))?;
        if !account.active {
            return Err(AccountSecurityError::Validation(
                "The account is switched off: switch it on before sending a password mail".to_string(),
            ));
        }
        self.mail_link_of(&account.username, kind).await
    }

    /// Asks the identity provider for a link and mails it; when the mail cannot go out the link
    /// comes back in the result so the administrator can hand it over
    pub(super) async fn mail_link_of(
        &self,
        username: &str,
        kind: PasswordMailKind,
    ) -> Result<PasswordMailResult, AccountSecurityError> {
        let identity = self
            .identity
            .as_ref()
            .ok_or_else(|| AccountSecurityError::Identity("no identity provider is configured".to_string()))?;
        let email = identity
            .find_email(username)
            .await
            .map_err(|e| AccountSecurityError::Identity(e.to_string()))?
            .ok_or_else(|| AccountSecurityError::NotFound(format!("no e-mail address for {username}")))?;
        let link = identity
            .create_recovery_link(username)
            .await
            .map_err(|e| AccountSecurityError::Identity(e.to_string()))?;

        let templates = self.mail_templates().await;
        let mut message = password_mail_message(kind, &email, username, &link.url, LINK_VALID_HOURS, &templates);
        message.requested_by = application_core::mail_requester::current();
        match self.mail.send(message).await {
            Ok(()) => {
                tracing::info!(account = %username, ?kind, "password mail sent");
                Ok(PasswordMailResult { sent: true, sent_to: Some(mask_email(&email)), mail_error: None, recovery_link: None })
            }
            Err(e) => {
                tracing::warn!(account = %username, ?kind, "password mail not sent: {e}");
                Ok(PasswordMailResult {
                    sent: false,
                    sent_to: Some(mask_email(&email)),
                    mail_error: Some(e.to_string()),
                    recovery_link: Some(link.url),
                })
            }
        }
    }

    /// The holder tried to sign in while a change is required
    pub(super) async fn resend_required_mail(&self, identifier: &str) {
        let name = account_key(identifier);
        let resend_minutes = u64::from(self.settings().await.mail_resend_minutes);
        let key = format!("authguard:pwmail:{name}");
        // At most one mail per interval, so a loop of sign-in attempts cannot flood the mailbox
        if resend_minutes > 0 {
            match self.cache.exists(&key).await {
                Ok(true) => return,
                Ok(false) => {}
                Err(e) => {
                    tracing::warn!("password mail throttle unreadable, not sending: {e}");
                    return;
                }
            }
            if let Err(e) = self.cache.set(&key, "1", Some(resend_minutes * 60)).await {
                tracing::warn!("password mail throttle not stored, not sending: {e}");
                return;
            }
        }
        match self.queries.find_account_by_identifier(identifier).await {
            Ok(Some(account)) if account.active && account.must_change_password => {
                if let Err(e) = self.mail_link_of(&account.username, PasswordMailKind::Required).await {
                    tracing::warn!(account = %account.username, "required-change mail failed: {e}");
                }
            }
            Ok(_) => {}
            Err(e) => tracing::warn!("account lookup for the required-change mail failed: {e}"),
        }
    }
}
