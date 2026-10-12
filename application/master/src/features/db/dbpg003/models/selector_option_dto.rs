//! DBPG003 models: branch, department and prefix options for the cascading selectors

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Branch option for the cascading company → branch → department selectors
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchOptionDto {
    pub id: Uuid,
    pub company_id: Uuid,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
    pub active: bool,
}

/// Department option for the cascading selectors
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DepartmentOptionDto {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Uuid,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
    pub active: bool,
}

/// Name prefix (`prefix` master lookup: นาย / นาง / ...)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrefixOptionDto {
    pub id: Uuid,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}
