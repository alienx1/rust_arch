//! Password mails, the forced password change, the password policy and the sign-in lock

use application_core::traits::{CacheService, InMemoryCache, MailService};
use application_master::features::su::account_security::{AccountSecurityCommands, AccountSecurityExecutor, DbAccountSecurityQueries};
use std::sync::Arc;
use tracing::warn;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    // Failed sign-in counters live in Valkey, or in this process while Valkey is down
    let lock_store: Arc<dyn CacheService> = match &cx.cache {
        Some(c) => Arc::new(c.clone()),
        None => Arc::new(InMemoryCache::new()),
    };
    // Mail goes to the notification service through Redpanda (it sends it and retries); without a
    // broker client this process sends it itself
    let mail: Arc<dyn MailService> = match &cx.broker {
        Some(events) => Arc::new(infrastructure::QueuedMailService::new(events.clone())),
        None => {
            warn!("Mail queue (Redpanda) not available, sending mail directly");
            Arc::new(infrastructure::ConfiguredMailService::new(cx.db.clone()))
        }
    };
    let mut executor = AccountSecurityExecutor::new(Arc::new(DbAccountSecurityQueries::new(cx.db.clone())), lock_store)
        .with_cache(cx.caches.clone())
        .with_mail(mail);
    if let Some(k) = &cx.kratos {
        executor = executor.with_identity(Arc::new(k.clone())).with_session_revocation(Arc::new(k.clone()));
    }
    let commands: Arc<dyn AccountSecurityCommands> = Arc::new(executor);
    services.insert(commands);
}
