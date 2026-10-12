//! DBPG006 one page of label entries

use serde::{Deserialize, Serialize};

use super::label_entry_dto::LabelEntryDto;

/// `total` counts distinct `(category, key)` entries, not rows
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabelPageDto {
    pub items: Vec<LabelEntryDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}
