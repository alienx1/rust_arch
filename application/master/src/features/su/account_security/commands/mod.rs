//! Account security commands

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::su::account_security::errors::AccountSecurityError;
use crate::features::su::account_security::models::{LoginState, PasswordMailKind, PasswordMailResult};

mod account_security_executor;
mod force_password_change;
mod login_lock;
mod password_state;
mod send_password_mail;

pub use account_security_executor::AccountSecurityExecutor;

#[async_trait]
pub trait AccountSecurityCommands: Send + Sync {
    /// Whether the account behind a login identifier may try to sign in now (locked after too
    /// many failures). Unknown identifiers are counted like known ones, so the answer never tells
    /// whether an account exists.
    async fn login_state(&self, identifier: &str) -> Result<LoginState, AccountSecurityError>;

    /// Counts a failed sign-in; the answer says whether this one locked the account
    async fn record_login_failure(&self, identifier: &str) -> Result<LoginState, AccountSecurityError>;

    /// A successful sign-in clears the failures
    async fn record_login_success(&self, identifier: &str) -> Result<(), AccountSecurityError>;

    /// The user name of the account a sign-in identifier belongs to. The sign-in e-mail can differ
    /// from the user name (it follows the employee's contacts), so an identifier that is neither a
    /// user name nor `<user name>@…` is resolved through the identity provider. An identifier that
    /// cannot be resolved is returned as it is.
    async fn account_name(&self, identifier: &str) -> String {
        identifier.to_string()
    }

    /// The account must set a new password before it may sign in
    async fn password_change_required(&self, identifier: &str) -> Result<bool, AccountSecurityError>;

    /// The identity provider reports a new password: the account may sign in again
    async fn mark_password_changed(&self, identifier: &str) -> Result<(), AccountSecurityError>;

    /// The holder tried to sign in while a change is required: mails the link again, at most once
    /// per `security.password.mail_resend_minutes`. Never fails the sign-in attempt.
    async fn notify_password_required(&self, identifier: &str);

    /// E-mails the account holder a link to set a password
    async fn send_password_mail(&self, user_id: Uuid, kind: PasswordMailKind) -> Result<PasswordMailResult, AccountSecurityError>;

    /// Requires a new password: the account cannot sign in until it is set, its sessions end, and
    /// the link is e-mailed
    async fn force_password_change(&self, user_id: Uuid) -> Result<PasswordMailResult, AccountSecurityError>;
}
