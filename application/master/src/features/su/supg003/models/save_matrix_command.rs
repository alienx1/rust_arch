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
    /// Row version of the group the matrix was loaded with; a matrix saved meanwhile (or a group
    /// changed) is a version conflict. The group's version moves up by one when the matrix changes.
    pub row_version: i64,
}
