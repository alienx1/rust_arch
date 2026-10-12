//! Command DTO to create a province

use serde::{Deserialize, Serialize};

/// Command DTO to create a province (country = the active country)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateProvinceCommand {
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}
