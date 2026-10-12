//! DBPG002 models: branch with company context and localized names

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Branch with parent company context and localized names
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub company_id: Uuid,
    pub company_code: String,
    pub company_name: String,
    pub code: String,
    pub master_lookup_id: Uuid,
    pub branch_type_code: String,
    pub branch_type_name: String,
    pub is_headquarter: bool,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub department_count: u64,
    pub row_version: i64,
}
