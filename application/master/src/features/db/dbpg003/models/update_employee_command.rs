//! DBPG003 models: command to update an employee

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Command DTO to update an employee (may move to another department)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateEmployeeCommand {
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
    pub row_version: i64,
}
