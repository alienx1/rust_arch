//! ALPG001 the validated filter handed to the queries

use chrono::{DateTime, Utc};

/// A [`super::ProcessFilterQuery`] after validation: page >= 1, limit clamped, times parsed
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcessSearch {
    pub status: Option<String>,
    pub kind: Option<String>,
    pub user: Option<String>,
    /// Inclusive
    pub from: Option<DateTime<Utc>>,
    /// Exclusive
    pub to: Option<DateTime<Utc>>,
    pub page: u64,
    pub limit: u64,
}
