//! NTPG000 limits

pub const DEFAULT_LIST_LIMIT: u64 = 20;
/// A user keeps at most this many notifications; the oldest go when a new one arrives
pub const MAX_NOTIFICATIONS_PER_USER: u64 = 300;
pub const MAX_LIST_LIMIT: u64 = MAX_NOTIFICATIONS_PER_USER;
