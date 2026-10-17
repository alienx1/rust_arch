//! DBPG003 models: employee with organizational context and localized person name

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Employee with organizational context and localized person name
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmployeeDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub company_id: Uuid,
    pub company_code: String,
    pub company_name: String,
    pub branch_id: Uuid,
    pub branch_code: String,
    pub branch_name: String,
    pub department_id: Uuid,
    pub department_code: String,
    pub department_name: String,
    pub person_name_id: Uuid,
    pub prefix_id: Uuid,
    pub prefix_name: String,
    pub code: String,
    /// Prefix + name in the request language (the other language when that name is empty)
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
    /// Whether a `su_user` login account is linked to this employee
    pub has_user_account: bool,
    /// Photo URL for `<img src>` (`None` = no image)
    #[serde(default)]
    pub image_url: Option<String>,
    pub row_version: i64,
}
