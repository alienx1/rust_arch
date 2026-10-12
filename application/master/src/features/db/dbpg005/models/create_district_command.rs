//! Command DTO to create a district

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Command DTO to create a district under a province
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateDistrictCommand {
    pub province_id: Uuid,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}
