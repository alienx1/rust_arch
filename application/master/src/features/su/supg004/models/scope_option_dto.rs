//! SUPG004 scope option DTOs for the assignment picker

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An organization / company / branch / department an assignment can point at
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScopeOptionDto {
    pub id: Uuid,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
    /// The unit one level up (company → organization, branch → company, department → branch)
    pub parent_id: Option<Uuid>,
}

/// Picker options, one list per scope level
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScopeOptionsDto {
    pub organizations: Vec<ScopeOptionDto>,
    pub companies: Vec<ScopeOptionDto>,
    pub branches: Vec<ScopeOptionDto>,
    pub departments: Vec<ScopeOptionDto>,
}
