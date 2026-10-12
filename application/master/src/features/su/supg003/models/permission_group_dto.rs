//! SUPG003 permission group DTO

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A permission group (no scope: scopes belong to user assignments) with how many users / grants it has.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionGroupDto {
    pub id: Uuid,
    pub code: String,
    pub active: bool,
    pub user_count: u64,
    /// Programs the group may READ
    pub readable_programs: u64,
    pub is_protected: bool,
    pub row_version: i64,
}
