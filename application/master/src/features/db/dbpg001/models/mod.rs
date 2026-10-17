//! DBPG001 models: write commands plus the read DTOs shared with `dbpg000`

pub use crate::features::db::dbpg000::models::{
    CompanyDto, CompanyTypeLookupDto, ENGLISH_LANGUAGE_ID, ImageOwner, MAX_IMAGE_BYTES, OrganizationDto,
    StoredImage, THAI_LANGUAGE_ID, content_type_of_key, image_url, new_image_key, sniff_image,
    db_company, db_company_label, db_organization, db_organization_label,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Command DTO to create a new organization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateOrganizationCommand {
    pub code: String,
    pub registration_number: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}

/// Command DTO to update an existing organization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateOrganizationCommand {
    pub code: String,
    pub registration_number: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}

/// Command DTO to create a new company
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateCompanyCommand {
    pub organization_id: Uuid,
    pub code: String,
    pub master_lookup_id: Uuid,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}

/// Command DTO to update an existing company
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateCompanyCommand {
    pub organization_id: Uuid,
    pub code: String,
    pub master_lookup_id: Uuid,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}
