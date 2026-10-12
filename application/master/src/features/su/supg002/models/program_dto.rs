//! SUPG002 program DTO and its page wrapper

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A catalogue program (`db_program`) with its labels and where it sits
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgramDto {
    pub id: Uuid,
    pub system_id: Uuid,
    pub system_code: String,
    pub module_id: Uuid,
    pub module_code: String,
    /// Parent folder (`db_program.program_id`)
    pub parent_id: Option<Uuid>,
    pub parent_code: String,
    pub code: String,
    /// SPA route; `None` for a folder without a page of its own
    pub path: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub child_count: u64,
    /// Cannot be switched off (see [`PROTECTED_PROGRAM_CODES`])
    ///
    /// [`PROTECTED_PROGRAM_CODES`]: super::PROTECTED_PROGRAM_CODES
    pub is_protected: bool,
    pub row_version: i64,
}

/// One page of programs
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgramPageDto {
    pub items: Vec<ProgramDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}
