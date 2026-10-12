//! SUPG004 assignment output: a permission group an account holds at one scope unit

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserAssignmentDto {
    pub group_id: Uuid,
    pub group_code: String,
    pub scope_level: String,
    pub scope_id: Uuid,
    pub scope_code: String,
    /// Name of the unit in the request language
    pub scope_name: String,
    /// Codes from the organization down to the unit, joined with ` › `
    pub scope_path: String,
}
