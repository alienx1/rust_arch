//! SUPG002 form selector option DTOs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Selector option: a system
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemOptionDto {
    pub id: Uuid,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}

/// Selector option: a module
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleOptionDto {
    pub id: Uuid,
    pub system_id: Uuid,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}

/// Selector option: a program that can be a parent folder
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgramOptionDto {
    pub id: Uuid,
    pub system_id: Uuid,
    pub module_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}

/// Everything the program form needs to fill its selectors
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgramOptionsDto {
    pub systems: Vec<SystemOptionDto>,
    pub modules: Vec<ModuleOptionDto>,
    pub programs: Vec<ProgramOptionDto>,
}
