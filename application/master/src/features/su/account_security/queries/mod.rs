//! Account security queries: the account row and the settings

use std::collections::HashMap;

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::su::account_security::errors::AccountSecurityError;

pub mod db_account_security_queries;

pub use db_account_security_queries::DbAccountSecurityQueries;

/// The part of an account the security actions need
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountRef {
    pub id: Uuid,
    pub username: String,
    pub active: bool,
    pub must_change_password: bool,
}

#[async_trait]
pub trait AccountSecurityQueries: Send + Sync {
    async fn find_account(&self, id: Uuid) -> Result<Option<AccountRef>, AccountSecurityError>;

    /// By the login identifier (the username, or an e-mail whose local part is the username)
    async fn find_account_by_identifier(&self, identifier: &str) -> Result<Option<AccountRef>, AccountSecurityError>;

    /// Sets `must_change_password` (the row version stays: an open edit form can still be saved).
    /// `false` when the account does not exist.
    async fn set_must_change_password(&self, id: Uuid, value: bool) -> Result<bool, AccountSecurityError>;

    /// `su_configuration` rows of category `security`: key -> value
    async fn security_configuration(&self) -> Result<HashMap<String, String>, AccountSecurityError>;

    /// `su_configuration` rows of category `mail_template` (subject / html / text of each password
    /// mail): key -> value. The default has none (the built-in texts are used).
    async fn mail_template_configuration(&self) -> Result<HashMap<String, String>, AccountSecurityError> {
        Ok(HashMap::new())
    }
}
