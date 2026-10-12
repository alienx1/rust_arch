//! DBPG006 category with its number of entries

use serde::{Deserialize, Serialize};

/// `entries` counts distinct keys of the category
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabelCategoryDto {
    pub category: String,
    pub entries: u64,
}
