//! Command DTO to update a province / district / sub-district

use serde::{Deserialize, Serialize};

/// Command DTO to update a province / district / sub-district. Code and parents are fixed:
/// addresses and the levels below reference them.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateGeoNodeCommand {
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}
