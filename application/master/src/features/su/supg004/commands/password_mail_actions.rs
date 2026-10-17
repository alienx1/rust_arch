//! SUPG004 password actions: mail a link, force a change (the work is the account security feature's)

use uuid::Uuid;

use super::supg004_executor::Supg004CommandExecutor;
use crate::features::su::account_security::{AccountSecurityError, PasswordMailKind, PasswordMailResult};
use crate::features::su::supg004::errors::Supg004Error;

fn failed(e: AccountSecurityError) -> Supg004Error {
    match e {
        AccountSecurityError::NotFound(m) => Supg004Error::NotFound(m),
        AccountSecurityError::Validation(m) => Supg004Error::Validation(m),
        AccountSecurityError::Identity(m) => Supg004Error::IdentityProvider(m),
        AccountSecurityError::Database(m) => Supg004Error::Database(m),
    }
}

impl Supg004CommandExecutor {
    pub(super) async fn send_password_mail(&self, id: Uuid) -> Result<PasswordMailResult, Supg004Error> {
        self.security()?.send_password_mail(id, PasswordMailKind::Reset).await.map_err(failed)
    }

    pub(super) async fn force_password_change(&self, id: Uuid) -> Result<PasswordMailResult, Supg004Error> {
        self.security()?.force_password_change(id).await.map_err(failed)
    }
}
