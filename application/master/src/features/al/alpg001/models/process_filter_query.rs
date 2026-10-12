//! ALPG001 list filter (query string)

use serde::{Deserialize, Serialize};

/// `status` is one of `queued` / `running` / `succeeded` / `failed`, `kind` a process kind,
/// `user` the requester's username, `from` / `to` an RFC 3339 range on the request time
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProcessFilterQuery {
    pub status: Option<String>,
    pub kind: Option<String>,
    pub user: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}
