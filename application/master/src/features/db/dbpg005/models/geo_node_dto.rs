//! Geography node DTO (province / district / sub-district) and its list page

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A province, district or sub-district (told apart by `level`) with its parents' names
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeoNodeDto {
    pub id: Uuid,
    pub level: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    /// Parent province (districts and sub-districts)
    pub province_id: Option<Uuid>,
    pub province_name: String,
    /// Parent district (sub-districts)
    pub district_id: Option<Uuid>,
    pub district_name: String,
    /// Postal codes of the sub-district (sub-districts only)
    pub postal_codes: Vec<String>,
    pub row_version: i64,
}

/// One page of provinces / districts / sub-districts
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeoPageDto {
    pub items: Vec<GeoNodeDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}
