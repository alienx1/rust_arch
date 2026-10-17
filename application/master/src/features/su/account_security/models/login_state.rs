//! The sign-in lock

use serde::{Deserialize, Serialize};

/// Whether an account may attempt to sign in right now
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoginState {
    Open,
    /// Too many failed sign-ins: locked for this many more seconds
    Locked { retry_after_secs: u64 },
}

/// How many failures within how long lock for how long
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThrottleRule {
    pub max_failures: u32,
    pub window_secs: u64,
    pub lock_secs: u64,
}
