//! SUPG001 system DTO

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A system with its localized names and what hangs under it
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemDto {
    pub id: Uuid,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub module_count: u64,
    pub program_count: u64,
    /// Cannot be switched off (see [`PROTECTED_SYSTEM_CODES`])
    ///
    /// [`PROTECTED_SYSTEM_CODES`]: super::PROTECTED_SYSTEM_CODES
    pub is_protected: bool,
    pub row_version: i64,
}
