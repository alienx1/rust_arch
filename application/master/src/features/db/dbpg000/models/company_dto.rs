//! DBPG000 company DTOs: detail row and company type lookup

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Company detailed DTO with parent organization context and localized names
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub organization_code: String,
    pub organization_name: String,
    pub code: String,
    pub master_lookup_id: Uuid,
    pub company_type_name: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    /// Logo URL for `<img src>` (`None` = no image)
    #[serde(default)]
    pub image_url: Option<String>,
    pub row_version: i64,
}

/// Company Type Master Lookup DTO
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyTypeLookupDto {
    pub id: Uuid,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}
