//! DBPG006 label entry: `(category, key)` with its translations

use serde::{Deserialize, Serialize};

use super::label_value_dto::LabelValueDto;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabelEntryDto {
    pub category: String,
    pub key: String,
    pub values: Vec<LabelValueDto>,
}
