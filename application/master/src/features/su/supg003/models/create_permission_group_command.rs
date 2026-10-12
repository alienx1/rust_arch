//! SUPG003 create-group command

use serde::{Deserialize, Serialize};

/// Command DTO to create a group (the code is fixed afterwards: users are bound to it)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatePermissionGroupCommand {
    pub code: String,
    pub active: bool,
}
