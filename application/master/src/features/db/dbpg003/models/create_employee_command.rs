//! DBPG003 models: command to create an employee

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Command DTO to create an employee. Organization, company and branch follow the department.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateEmployeeCommand {
    pub department_id: Uuid,
    pub prefix_id: Uuid,
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
