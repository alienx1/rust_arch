//! DBPG000 organization DTOs: detail row and KPI summary

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Organization detailed DTO
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationDto {
    pub id: Uuid,
    pub code: String,
    pub registration_number: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub company_count: u64,
    pub row_version: i64,
}

/// High-level organization and company statistics summary
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationSummaryDto {
    pub total_organizations: u64,
    pub active_organizations: u64,
    pub total_companies: u64,
    pub active_companies: u64,
}
