//! SUPG004 update-account command

use serde::{Deserialize, Serialize};
use super::AssignmentCommand;

/// Command DTO to move an account to another group and/or switch it on/off. The username and
/// employee are fixed (the identity provider and history reference them).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateUserCommand {
    /// Replaces all assignments of the account (at least one)
    pub assignments: Vec<AssignmentCommand>,
    pub active: bool,
    pub row_version: i64,
}
