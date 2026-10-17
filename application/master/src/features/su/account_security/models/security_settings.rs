//! Security settings read from `su_configuration` (category `security`), with safe defaults

use std::collections::HashMap;

use super::login_state::ThrottleRule;

/// The keys of `su_configuration` this feature reads
pub const SETTING_KEYS: [&str; 4] = [
    "security.login.max_failed_attempts",
    "security.login.attempt_window_minutes",
    "security.login.lock_minutes",
    "security.password.mail_resend_minutes",
];

/// Settings of the sign-in lock and the password mails
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SecuritySettings {
    /// Failed sign-ins within the window after which the account is locked (`0` = lock off)
    pub max_failed_attempts: u32,
    pub attempt_window_minutes: u32,
    pub lock_minutes: u32,
    /// Least minutes between two password mails sent because the account tried to sign in
    pub mail_resend_minutes: u32,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self { max_failed_attempts: 5, attempt_window_minutes: 15, lock_minutes: 15, mail_resend_minutes: 5 }
    }
}

impl SecuritySettings {
    /// Settings from `key -> value` rows; a missing or unreadable value keeps its default, and a
    /// window / lock of zero minutes is raised to one so a typo cannot disable the lock by accident
    /// (the lock is switched off only with `max_failed_attempts = 0`)
    pub fn from_rows(rows: &HashMap<String, String>) -> Self {
        let defaults = Self::default();
        let number = |key: &str, default: u32| {
            rows.get(key).and_then(|v| v.trim().parse::<u32>().ok()).unwrap_or(default)
        };
        Self {
            max_failed_attempts: number(SETTING_KEYS[0], defaults.max_failed_attempts),
            attempt_window_minutes: number(SETTING_KEYS[1], defaults.attempt_window_minutes).max(1),
            lock_minutes: number(SETTING_KEYS[2], defaults.lock_minutes).max(1),
            mail_resend_minutes: number(SETTING_KEYS[3], defaults.mail_resend_minutes),
        }
    }

    /// The rule the lock applies; `None` when the lock is switched off
    pub fn throttle_rule(&self) -> Option<ThrottleRule> {
        (self.max_failed_attempts > 0).then(|| ThrottleRule {
            max_failures: self.max_failed_attempts,
            window_secs: u64::from(self.attempt_window_minutes) * 60,
            lock_secs: u64::from(self.lock_minutes) * 60,
        })
    }
}
