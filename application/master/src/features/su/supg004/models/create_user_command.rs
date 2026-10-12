//! SUPG004 create-account command

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AssignmentCommand;

/// Command DTO to create an account
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateUserCommand {
    pub employee_id: Uuid,
    pub username: String,
    /// Sign-in / recovery email at the identity provider (not stored locally)
    pub email: String,
    /// At least one; the first one's group is the primary group of the account
    pub assignments: Vec<AssignmentCommand>,
    pub active: bool,
}
