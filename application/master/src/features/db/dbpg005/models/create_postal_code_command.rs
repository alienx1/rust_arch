//! Command DTO to create a postal code

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Command DTO to create a postal code. Give the most specific area; the levels above it are
/// taken from that area, so they can never disagree.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatePostalCodeCommand {
    pub province_id: Uuid,
    pub district_id: Option<Uuid>,
    pub sub_district_id: Option<Uuid>,
    pub postal_code: String,
    pub active: bool,
}
