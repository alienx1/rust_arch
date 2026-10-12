//! DBPG004 DTOs: reference data (`db_master_lookup`) read models and write payloads

use serde::{Deserialize, Serialize};

use crate::pages::db::dbpg000::models::deserialize_id;

/// One lookup value with its Thai / English labels
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LookupDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub category: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    /// System-core row: cannot be deactivated
    pub is_protected: bool,
    pub row_version: i64,
}

/// A category with its row counts
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LookupCategoryDto {
    pub category: String,
    pub total: u64,
    pub active: u64,
    pub is_protected: bool,
}

/// One page of lookups
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LookupPageDto {
    pub items: Vec<LookupDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

/// Reference data KPI summary
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Dbpg004SummaryDto {
    pub total_lookups: u64,
    pub active_lookups: u64,
    pub total_categories: u64,
}

/// Input payload to create a lookup value (a new category name creates the category)
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateLookupInput {
    pub category: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}

/// Input payload to update a lookup value (category and code are fixed)
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateLookupInput {
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}
