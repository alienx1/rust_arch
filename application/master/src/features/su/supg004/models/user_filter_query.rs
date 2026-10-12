//! SUPG004 account list filter and paging

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Filter + paging for the account list (`search` matches username, employee code and names)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserFilterQuery {
    pub group_id: Option<Uuid>,
    pub search: Option<String>,
    pub active: Option<bool>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}
