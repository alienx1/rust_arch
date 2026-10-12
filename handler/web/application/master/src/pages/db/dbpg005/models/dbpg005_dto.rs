//! DBPG005 DTOs: geography (provinces, districts, sub-districts, postal codes)

use serde::{Deserialize, Serialize};
use web_package::core::i18n::t_or;

use crate::pages::db::dbpg000::models::{
    deserialize_id, deserialize_opt_id, serialize_id, serialize_opt_id,
};

/// The four levels the page manages
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeoLevel {
    Province,
    District,
    SubDistrict,
    PostalCode,
}

impl GeoLevel {
    pub const ALL: [GeoLevel; 4] =
        [GeoLevel::Province, GeoLevel::District, GeoLevel::SubDistrict, GeoLevel::PostalCode];

    /// Segment used in routes and form URLs
    pub fn slug(self) -> &'static str {
        match self {
            GeoLevel::Province => "province",
            GeoLevel::District => "district",
            GeoLevel::SubDistrict => "sub-district",
            GeoLevel::PostalCode => "postal-code",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|l| l.slug() == slug)
    }

    pub fn label(self) -> String {
        match self {
            GeoLevel::Province => t_or("dbpg005.level.province", "จังหวัด"),
            GeoLevel::District => t_or("dbpg005.level.district", "อำเภอ / เขต"),
            GeoLevel::SubDistrict => t_or("dbpg005.level.sub_district", "ตำบล / แขวง"),
            GeoLevel::PostalCode => t_or("dbpg005.level.postal_code", "รหัสไปรษณีย์"),
        }
    }
}

/// A province, district or sub-district (told apart by `level`)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeoNodeDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub level: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    #[serde(default, deserialize_with = "deserialize_opt_id")]
    pub province_id: Option<String>,
    pub province_name: String,
    #[serde(default, deserialize_with = "deserialize_opt_id")]
    pub district_id: Option<String>,
    pub district_name: String,
    pub postal_codes: Vec<String>,
    pub row_version: i64,
}

impl GeoNodeDto {
    pub fn display_name(&self) -> String {
        let name = if self.name_th.is_empty() { &self.name_en } else { &self.name_th };
        format!("{name} ({})", self.code)
    }
}

/// One page of provinces / districts / sub-districts
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GeoPageDto {
    pub items: Vec<GeoNodeDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

/// A postal code with the area it belongs to
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PostalCodeDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub postal_code: String,
    pub active: bool,
    #[serde(deserialize_with = "deserialize_id")]
    pub province_id: String,
    pub province_name: String,
    #[serde(default, deserialize_with = "deserialize_opt_id")]
    pub district_id: Option<String>,
    pub district_name: String,
    #[serde(default, deserialize_with = "deserialize_opt_id")]
    pub sub_district_id: Option<String>,
    pub sub_district_name: String,
    pub row_version: i64,
}

/// One page of postal codes
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PostalPageDto {
    pub items: Vec<PostalCodeDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

/// Row counts per level
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Dbpg005SummaryDto {
    pub provinces: u64,
    pub districts: u64,
    pub sub_districts: u64,
    pub postal_codes: u64,
}

/// Values of the geography form; the save logic picks what the level needs
#[derive(Clone, Debug, Default)]
pub struct GeoFormInput {
    pub province_id: String,
    pub district_id: String,
    pub sub_district_id: String,
    pub code: String,
    pub postal_code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateProvinceInput {
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateDistrictInput {
    #[serde(serialize_with = "serialize_id")]
    pub province_id: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateSubDistrictInput {
    #[serde(serialize_with = "serialize_id")]
    pub district_id: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateGeoNodeInput {
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatePostalCodeInput {
    #[serde(serialize_with = "serialize_id")]
    pub province_id: String,
    #[serde(serialize_with = "serialize_opt_id")]
    pub district_id: Option<String>,
    #[serde(serialize_with = "serialize_opt_id")]
    pub sub_district_id: Option<String>,
    pub postal_code: String,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdatePostalCodeInput {
    pub postal_code: String,
    pub active: bool,
    pub row_version: i64,
}
