//! DBPG002 DTOs: branches & departments (read models and write payloads)

use serde::{Deserialize, Serialize};
use web_package::core::i18n::localized;

use crate::pages::db::dbpg000::models::{deserialize_id, deserialize_opt_id, serialize_id, serialize_opt_id};

/// Company option for the company filter / parent selector
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyOptionDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub organization_id: String,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}

impl CompanyOptionDto {
    pub fn display_name(&self) -> String {
        let name = localized(&self.name_th, &self.name_en);
        format!("{name} ({})", self.code)
    }
}

/// Permission group option for a department's default group
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionGroupOptionDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub code: String,
}

/// Branch type option (HEAD / BRANCH)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchTypeOptionDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}

/// Branch with parent company context
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub organization_id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub company_id: String,
    pub company_code: String,
    pub company_name: String,
    pub code: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub master_lookup_id: String,
    pub branch_type_code: String,
    pub branch_type_name: String,
    pub is_headquarter: bool,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub department_count: u64,
    pub row_version: i64,
}

impl BranchDto {
    pub fn display_name(&self) -> String {
        let name = localized(&self.name_th, &self.name_en);
        format!("{name} ({})", self.code)
    }
}

/// Department with parent branch/company context
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DepartmentDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub organization_id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub company_id: String,
    pub company_code: String,
    pub company_name: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub branch_id: String,
    pub branch_code: String,
    pub branch_name: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    /// Permission group the accounts of the department's employees get at the branch
    #[serde(default, deserialize_with = "deserialize_opt_id")]
    pub default_permission_group_id: Option<String>,
    #[serde(default)]
    pub default_permission_group_code: Option<String>,
    pub row_version: i64,
}

/// Branch & department KPI summary
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Dbpg002SummaryDto {
    pub total_branches: u64,
    pub active_branches: u64,
    pub total_departments: u64,
    pub active_departments: u64,
}

/// Input payload to create a branch
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateBranchInput {
    #[serde(serialize_with = "serialize_id")]
    pub company_id: String,
    pub code: String,
    #[serde(serialize_with = "serialize_id")]
    pub master_lookup_id: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}

/// Input payload to update a branch
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateBranchInput {
    pub code: String,
    #[serde(serialize_with = "serialize_id")]
    pub master_lookup_id: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}

/// Input payload to create a department
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateDepartmentInput {
    #[serde(serialize_with = "serialize_id")]
    pub branch_id: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    /// Permission group the accounts of the department's employees get at the branch (optional)
    #[serde(default, serialize_with = "serialize_opt_id")]
    pub default_permission_group_id: Option<String>,
}

/// Input payload to update a department
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateDepartmentInput {
    #[serde(serialize_with = "serialize_id")]
    pub branch_id: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    #[serde(default, serialize_with = "serialize_opt_id")]
    pub default_permission_group_id: Option<String>,
    pub row_version: i64,
}
