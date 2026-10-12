//! DBPG004 lookup DTOs: lookup value, category counts, page and KPI summary

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One lookup value with its Thai / English labels
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LookupDto {
    pub id: Uuid,
    pub category: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    /// System-core row: cannot be deactivated (see [`PROTECTED_CATEGORIES`](super::PROTECTED_CATEGORIES))
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LookupPageDto {
    pub items: Vec<LookupDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

/// Lookup KPI summary
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dbpg004SummaryDto {
    pub total_lookups: u64,
    pub active_lookups: u64,
    pub total_categories: u64,
}
