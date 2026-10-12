//! SUPG004 permission group option DTO

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A permission group an account can be placed in
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupOptionDto {
    pub id: Uuid,
    pub code: String,
    pub active: bool,
    /// Placing an account here needs APPROVE on SUPG004 (full-access group)
    pub needs_approval: bool,
}
