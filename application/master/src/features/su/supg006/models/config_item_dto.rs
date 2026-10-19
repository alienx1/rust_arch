//! One setting as the page shows it

use serde::{Deserialize, Serialize};

use super::config_settings::{ConfigGroup, ConfigKind};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigItemDto {
    pub key: String,
    pub group: ConfigGroup,
    pub kind: ConfigKind,
    /// Stored value; for a setting with no row yet, its default
    pub value: String,
    /// Value used when the row is missing or empty (numbers) or empty for a template, which then
    /// falls back to the built-in text
    pub default_value: String,
    /// Smallest / largest number accepted (numbers only)
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub description: Option<String>,
    /// Version of the row (0 = no row yet); send it back when saving
    pub row_version: i64,
}
