//! DBPG002 models: command to update a department

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Command DTO to update a department (may move to another branch)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateDepartmentCommand {
    pub branch_id: Uuid,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}
