//! SUPG004 assignment input: a permission group held at one scope unit

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One assignment of an account: `permission_group_id` held at the unit `scope_id` of `scope_level`
/// (organization | company | branch | department)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssignmentCommand {
    pub permission_group_id: Uuid,
    pub scope_level: String,
    pub scope_id: Uuid,
}
