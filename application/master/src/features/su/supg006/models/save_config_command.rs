//! Save command: the settings that changed

use serde::{Deserialize, Serialize};

/// One changed setting
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigChange {
    pub key: String,
    pub value: String,
    /// `row_version` the setting was loaded with (0 = it had no row)
    pub row_version: i64,
}

/// Command DTO: only the settings that changed, saved together or not at all
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveConfigCommand {
    pub changes: Vec<ConfigChange>,
}
