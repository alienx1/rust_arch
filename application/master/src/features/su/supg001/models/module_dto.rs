//! SUPG001 module DTO

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A module with its system and localized names
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleDto {
    pub id: Uuid,
    pub system_id: Uuid,
    pub system_code: String,
    pub system_name: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub program_count: u64,
    /// Cannot be switched off (see [`PROTECTED_MODULE_CODES`])
    ///
    /// [`PROTECTED_MODULE_CODES`]: super::PROTECTED_MODULE_CODES
    pub is_protected: bool,
    pub row_version: i64,
}
