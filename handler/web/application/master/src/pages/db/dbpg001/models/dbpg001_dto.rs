//! DBPG001 DTOs: write payloads for the organization/company forms.
//!
//! Read models are shared with `dbpg000`.

pub use crate::pages::db::dbpg000::models::{CompanyDto, CompanyTypeOptionDto, OrganizationDto};
use serde::{Deserialize, Serialize};

use crate::pages::db::dbpg000::models::serialize_id;

/// Input payload to create a new organization
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateOrgInput {
    pub code: String,
    pub registration_number: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}

/// Input payload to update an organization
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateOrgInput {
    pub code: String,
    pub registration_number: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}

/// Input payload to create a company
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateCompanyInput {
    #[serde(serialize_with = "serialize_id")]
    pub organization_id: String,
    pub code: String,
    #[serde(serialize_with = "serialize_id")]
    pub master_lookup_id: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}

/// Input payload to update a company
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateCompanyInput {
    #[serde(serialize_with = "serialize_id")]
    pub organization_id: String,
    pub code: String,
    #[serde(serialize_with = "serialize_id")]
    pub master_lookup_id: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}
