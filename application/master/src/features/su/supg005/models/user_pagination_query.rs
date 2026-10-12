//! User list pagination and search query

use serde::{Deserialize, Serialize};

/// พารามิเตอร์ Pagination และการค้นหาผู้ใช้สำหรับ SQL LIMIT / OFFSET
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UserPaginationQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub search: Option<String>,
    /// Session state (`ONLINE` / `OFFLINE`): only applied to the sample directory shown while the
    /// user table is empty; live sessions sit at the identity provider, so a real filter would
    /// have to ask it about every user
    pub status: Option<String>,
    /// Account state: `ACTIVE` / `SUSPENDED`; anything else = all
    #[serde(default)]
    pub account_status: Option<String>,
}
