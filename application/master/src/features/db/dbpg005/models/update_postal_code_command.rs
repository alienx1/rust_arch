//! Command DTO to update a postal code

use serde::{Deserialize, Serialize};

/// Command DTO to update a postal code value / status (its area is fixed)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdatePostalCodeCommand {
    pub postal_code: String,
    pub active: bool,
    pub row_version: i64,
}
