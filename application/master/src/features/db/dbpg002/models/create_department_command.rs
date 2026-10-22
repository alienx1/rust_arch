//! DBPG002 models: command to create a department

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Command DTO to create a department under a branch (organization/company come from the branch)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateDepartmentCommand {
    pub branch_id: Uuid,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    /// Permission group the accounts of the department's employees get at the branch (optional)
    #[serde(default)]
    pub default_permission_group_id: Option<Uuid>,
}
