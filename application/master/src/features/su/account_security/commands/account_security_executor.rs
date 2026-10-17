//! Account security executor: routes each `AccountSecurityCommands` method to its action file

use std::collections::HashMap;
use std::sync::Arc;

use application_core::traits::{
    CacheService, IdentityProvisioningPort, MailService, NoMailService, ReadCaches, SessionRevocationPort, cache_ns,
    cache_ttl,
};
use async_trait::async_trait;
use uuid::Uuid;

use super::AccountSecurityCommands;
use crate::features::su::account_security::errors::AccountSecurityError;
use crate::features::su::account_security::models::{
    LoginState, PasswordMailKind, PasswordMailResult, SecuritySettings,
};
use crate::features::su::account_security::queries::AccountSecurityQueries;

pub struct AccountSecurityExecutor {
    pub(super) queries: Arc<dyn AccountSecurityQueries>,
    /// Where the failure counters and the locks live (Valkey)
    pub(super) cache: Arc<dyn CacheService>,
    pub(super) identity: Option<Arc<dyn IdentityProvisioningPort>>,
    pub(super) revocation: Option<Arc<dyn SessionRevocationPort>>,
    pub(super) mail: Arc<dyn MailService>,
    /// Settings are read through Valkey (off until `with_cache`)
    pub(super) caches: ReadCaches,
}

impl AccountSecurityExecutor {
    pub fn new(queries: Arc<dyn AccountSecurityQueries>, cache: Arc<dyn CacheService>) -> Self {
        Self {
            queries,
            cache,
            identity: None,
            revocation: None,
            mail: Arc::new(NoMailService),
            caches: ReadCaches::disabled(),
        }
    }

    pub fn with_identity(mut self, identity: Arc<dyn IdentityProvisioningPort>) -> Self {
        self.identity = Some(identity);
        self
    }

    /// Ends the sessions of an account whose password change is forced
    pub fn with_session_revocation(mut self, revocation: Arc<dyn SessionRevocationPort>) -> Self {
        self.revocation = Some(revocation);
        self
    }

    pub fn with_mail(mut self, mail: Arc<dyn MailService>) -> Self {
        self.mail = mail;
        self
    }

    pub fn with_cache(mut self, caches: ReadCaches) -> Self {
        self.caches = caches;
        self
    }

    /// The mail templates in `su_configuration` (read through Valkey for a minute); none when they
    /// cannot be read, which falls back to the built-in texts
    pub(super) async fn mail_templates(&self) -> HashMap<String, String> {
        let cache = self.caches.ns(cache_ns::ACL).language_free();
        let key = cache.key("mail_templates", &());
        match cache.through_key(key, cache_ttl::ACL, || self.queries.mail_template_configuration()).await {
            Ok(rows) => rows,
            Err(e) => {
                tracing::warn!("mail templates not readable, using the built-in texts: {e}");
                HashMap::new()
            }
        }
    }

    /// The settings in `su_configuration` (read through Valkey for a minute)
    pub(super) async fn settings(&self) -> SecuritySettings {
        let cache = self.caches.ns(cache_ns::ACL).language_free();
        let key = cache.key("security_settings", &());
        let rows = cache.through_key(key, cache_ttl::ACL, || self.queries.security_configuration()).await;
        match rows {
            Ok(rows) => SecuritySettings::from_rows(&rows),
            Err(e) => {
                // Better the defaults than no lock at all
                tracing::warn!("security settings not readable, using the defaults: {e}");
                SecuritySettings::default()
            }
        }
    }
}

#[async_trait]
impl AccountSecurityCommands for AccountSecurityExecutor {
    async fn login_state(&self, identifier: &str) -> Result<LoginState, AccountSecurityError> {
        self.lock_state(identifier).await
    }

    async fn record_login_failure(&self, identifier: &str) -> Result<LoginState, AccountSecurityError> {
        self.count_failure(identifier).await
    }

    async fn record_login_success(&self, identifier: &str) -> Result<(), AccountSecurityError> {
        self.clear_failures(identifier).await
    }

    async fn account_name(&self, identifier: &str) -> String {
        // The identity provider knows whose sign-in this is; the e-mail's local part is only a guess
        // (someone could have an e-mail that starts with another account's user name)
        if let Some(identity) = &self.identity {
            match identity.find_username(identifier).await {
                Ok(Some(username)) => return username,
                Ok(None) => {}
                Err(e) => tracing::warn!("identity lookup for {identifier} failed: {e}"),
            }
        }
        match self.queries.find_account_by_identifier(identifier).await {
            Ok(Some(account)) => account.username,
            Ok(None) => identifier.to_string(),
            Err(e) => {
                tracing::warn!("account lookup for {identifier} failed: {e}");
                identifier.to_string()
            }
        }
    }

    async fn password_change_required(&self, identifier: &str) -> Result<bool, AccountSecurityError> {
        self.change_required(identifier).await
    }

    async fn mark_password_changed(&self, identifier: &str) -> Result<(), AccountSecurityError> {
        self.mark_changed(identifier).await
    }

    async fn notify_password_required(&self, identifier: &str) {
        self.resend_required_mail(identifier).await
    }

    async fn send_password_mail(&self, user_id: Uuid, kind: PasswordMailKind) -> Result<PasswordMailResult, AccountSecurityError> {
        self.mail_password_link(user_id, kind).await
    }

    async fn force_password_change(&self, user_id: Uuid) -> Result<PasswordMailResult, AccountSecurityError> {
        self.force_change(user_id).await
    }
}
