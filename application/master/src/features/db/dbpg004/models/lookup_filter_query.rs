//! DBPG004 lookup list filter

use serde::{Deserialize, Serialize};

/// Filter + paging for the lookup list (`search` matches code and labels)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LookupFilterQuery {
    pub category: Option<String>,
    pub search: Option<String>,
    pub active: Option<bool>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}
