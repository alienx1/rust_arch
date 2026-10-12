//! DBPG003 DTOs: employees (read models and write payloads)

use serde::{Deserialize, Serialize};
use web_package::core::i18n::{localized, t_or};

pub use crate::pages::db::dbpg002::models::CompanyOptionDto;
use crate::pages::db::dbpg000::models::{deserialize_id, serialize_id};

/// Branch option for the cascading selectors
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchOptionDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub company_id: String,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
    pub active: bool,
}

impl BranchOptionDto {
    pub fn display_name(&self) -> String {
        let name = localized(&self.name_th, &self.name_en);
        let off = if self.active {
            String::new()
        } else {
            format!(" ({})", t_or("dbpg003.option.inactive", "ปิดใช้งาน"))
        };
        format!("{name} ({}){off}", self.code)
    }
}

/// Department option for the cascading selectors
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DepartmentOptionDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub company_id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub branch_id: String,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
    pub active: bool,
}

impl DepartmentOptionDto {
    pub fn display_name(&self) -> String {
        let name = localized(&self.name_th, &self.name_en);
        let off = if self.active {
            String::new()
        } else {
            format!(" ({})", t_or("dbpg003.option.inactive", "ปิดใช้งาน"))
        };
        format!("{name} ({}){off}", self.code)
    }
}

/// Name prefix option (นาย / นาง / ...)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrefixOptionDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}

/// Employee with organizational context and localized person name
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmployeeDto {
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
    #[serde(deserialize_with = "deserialize_id")]
    pub department_id: String,
    pub department_code: String,
    pub department_name: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub person_name_id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub prefix_id: String,
    pub prefix_name: String,
    pub code: String,
    pub full_name: String,
    pub first_name_th: String,
    pub middle_name_th: String,
    pub last_name_th: String,
    pub nickname_th: String,
    pub first_name_en: String,
    pub middle_name_en: String,
    pub last_name_en: String,
    pub nickname_en: String,
    pub active: bool,
    pub has_user_account: bool,
    pub row_version: i64,
}

/// One page of employees
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EmployeePageDto {
    pub items: Vec<EmployeeDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

/// Employee KPI summary
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Dbpg003SummaryDto {
    pub total_employees: u64,
    pub active_employees: u64,
    pub with_user_account: u64,
}

/// Input payload to create an employee
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateEmployeeInput {
    #[serde(serialize_with = "serialize_id")]
    pub department_id: String,
    #[serde(serialize_with = "serialize_id")]
    pub prefix_id: String,
    pub code: String,
    pub active: bool,
    pub first_name_th: String,
    pub middle_name_th: String,
    pub last_name_th: String,
    pub nickname_th: String,
    pub first_name_en: String,
    pub middle_name_en: String,
    pub last_name_en: String,
    pub nickname_en: String,
}

/// Input payload to update an employee
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateEmployeeInput {
    #[serde(serialize_with = "serialize_id")]
    pub department_id: String,
    #[serde(serialize_with = "serialize_id")]
    pub prefix_id: String,
    pub code: String,
    pub active: bool,
    pub first_name_th: String,
    pub middle_name_th: String,
    pub last_name_th: String,
    pub nickname_th: String,
    pub first_name_en: String,
    pub middle_name_en: String,
    pub last_name_en: String,
    pub nickname_en: String,
    pub row_version: i64,
}
