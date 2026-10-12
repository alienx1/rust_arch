//! DBPG006 label list filter

use serde::{Deserialize, Serialize};

/// Filter + paging for the label list (`search` matches category, key and values)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LabelFilterQuery {
    pub category: Option<String>,
    pub search: Option<String>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}
