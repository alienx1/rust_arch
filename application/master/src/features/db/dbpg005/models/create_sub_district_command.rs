//! Command DTO to create a sub-district

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Command DTO to create a sub-district under a district
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateSubDistrictCommand {
    pub district_id: Uuid,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}
