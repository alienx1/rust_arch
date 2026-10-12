//! SUPG003 save-matrix command and its grants

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// What one program may do for the group
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrantDto {
    pub program_id: Uuid,
    pub read: bool,
    pub write: bool,
    pub approve: bool,
}

/// Command DTO to replace a group's matrix: programs not listed (or all-false) lose access
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveMatrixCommand {
    pub grants: Vec<GrantDto>,
}
