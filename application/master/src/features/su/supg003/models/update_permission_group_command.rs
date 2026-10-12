//! SUPG003 update-group command

use serde::{Deserialize, Serialize};

/// Command DTO to switch a group on / off (optimistic lock on `row_version`)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdatePermissionGroupCommand {
    pub active: bool,
    pub row_version: i64,
}
