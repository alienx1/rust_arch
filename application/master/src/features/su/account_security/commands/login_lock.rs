//! Sign-in lock: failures are counted per account name in Valkey; too many within the window
//! lock the name for a while. Counters and locks expire by themselves.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::account_security_executor::AccountSecurityExecutor;
use crate::features::su::account_security::errors::AccountSecurityError;
use crate::features::su::account_security::models::LoginState;

/// Prefix of every key of the lock (not under `arch:`, which `make cache-flush` clears: a flush
/// of the read caches must not unlock accounts)
const PREFIX: &str = "authguard:";

#[derive(Serialize, Deserialize)]
struct Failures {
    count: u32,
    /// Unix seconds of the first failure of the window
    since: u64,
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// The key the account is counted under: the name before any `@`, lower-case, so `admin`,
/// `Admin` and `admin@arch.local` are one account
pub(super) fn account_key(identifier: &str) -> String {
    identifier.trim().split('@').next().unwrap_or_default().to_lowercase()
}

fn cache_err(e: impl std::fmt::Display) -> AccountSecurityError {
    AccountSecurityError::Database(format!("lock store: {e}"))
}

impl AccountSecurityExecutor {
    pub(super) async fn lock_state(&self, identifier: &str) -> Result<LoginState, AccountSecurityError> {
        let key = format!("{PREFIX}lock:{}", account_key(identifier));
        match self.cache.get(&key).await.map_err(cache_err)? {
            Some(until) => {
                let until: u64 = until.parse().unwrap_or(0);
                let left = until.saturating_sub(now());
                Ok(if left > 0 { LoginState::Locked { retry_after_secs: left } } else { LoginState::Open })
            }
            None => Ok(LoginState::Open),
        }
    }

    pub(super) async fn count_failure(&self, identifier: &str) -> Result<LoginState, AccountSecurityError> {
        let Some(rule) = self.settings().await.throttle_rule() else {
            return Ok(LoginState::Open);
        };
        let name = account_key(identifier);
        if name.is_empty() {
            return Ok(LoginState::Open);
        }
        if let LoginState::Locked { retry_after_secs } = self.lock_state(identifier).await? {
            return Ok(LoginState::Locked { retry_after_secs });
        }
        let key = format!("{PREFIX}fail:{name}");
        let mut failures = match self.cache.get(&key).await.map_err(cache_err)? {
            Some(raw) => serde_json::from_str::<Failures>(&raw).unwrap_or(Failures { count: 0, since: now() }),
            None => Failures { count: 0, since: now() },
        };
        // A window that has run out starts again
        if now().saturating_sub(failures.since) >= rule.window_secs {
            failures = Failures { count: 0, since: now() };
        }
        failures.count += 1;

        if failures.count >= rule.max_failures {
            let until = now() + rule.lock_secs;
            self.cache
                .set(&format!("{PREFIX}lock:{name}"), &until.to_string(), Some(rule.lock_secs))
                .await
                .map_err(cache_err)?;
            self.cache.del(&key).await.map_err(cache_err)?;
            tracing::warn!(account = %name, minutes = rule.lock_secs / 60, "account locked after too many failed sign-ins");
            return Ok(LoginState::Locked { retry_after_secs: rule.lock_secs });
        }
        let left = rule.window_secs.saturating_sub(now().saturating_sub(failures.since)).max(1);
        let raw = serde_json::to_string(&failures).map_err(cache_err)?;
        self.cache.set(&key, &raw, Some(left)).await.map_err(cache_err)?;
        Ok(LoginState::Open)
    }

    pub(super) async fn clear_failures(&self, identifier: &str) -> Result<(), AccountSecurityError> {
        let name = account_key(identifier);
        self.cache.del(&format!("{PREFIX}fail:{name}")).await.map_err(cache_err)?;
        self.cache.del(&format!("{PREFIX}lock:{name}")).await.map_err(cache_err)
    }
}
