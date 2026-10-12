//! Postal code DTO and its list page

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A postal code with the area it belongs to
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PostalCodeDto {
    pub id: Uuid,
    pub postal_code: String,
    pub active: bool,
    pub province_id: Uuid,
    pub province_name: String,
    pub district_id: Option<Uuid>,
    pub district_name: String,
    pub sub_district_id: Option<Uuid>,
    pub sub_district_name: String,
    pub row_version: i64,
}

/// One page of postal codes
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PostalPageDto {
    pub items: Vec<PostalCodeDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}
