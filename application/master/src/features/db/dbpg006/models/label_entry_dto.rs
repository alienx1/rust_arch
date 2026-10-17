//! DBPG006 label entry: `(category, key)` with its translations

use serde::{Deserialize, Serialize};

use super::label_value_dto::LabelValueDto;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabelEntryDto {
    pub category: String,
    pub key: String,
    pub values: Vec<LabelValueDto>,
    /// Version of the entry: the sum of the versions of its translations. Send it back when saving;
    /// a save that finds a different sum is a version conflict.
    #[serde(default)]
    pub row_version: i64,
}
