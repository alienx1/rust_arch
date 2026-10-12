//! ALPG000 search filter (query string)

use serde::{Deserialize, Serialize};

/// Filter + paging of the audit list. `from` / `to` are RFC 3339; `kind` is one of `page_view`,
/// `click`, `action`, `api`; `user` matches user id, username or email; `status_class` is 2..=5;
/// `text` searches the whole event.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuditLogFilterQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub kind: Option<String>,
    pub user: Option<String>,
    pub program_code: Option<String>,
    pub method: Option<String>,
    pub status: Option<u16>,
    pub status_class: Option<u8>,
    pub path: Option<String>,
    pub text: Option<String>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}
