//! DBPG002 models: department with branch/company context and localized names

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Department with parent branch/company context and localized names
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DepartmentDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub company_id: Uuid,
    pub company_code: String,
    pub company_name: String,
    pub branch_id: Uuid,
    pub branch_code: String,
    pub branch_name: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}
