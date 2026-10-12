//! SUPG003 matrix DTOs: a group's program × action matrix

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::permission_group_dto::PermissionGroupDto;

/// One program row of the matrix (tree order, `depth` for indentation)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatrixRowDto {
    pub program_id: Uuid,
    pub system_code: String,
    pub module_code: String,
    pub program_code: String,
    pub name_th: String,
    pub name_en: String,
    pub parent_id: Option<Uuid>,
    pub depth: u32,
    pub is_folder: bool,
    pub program_active: bool,
    pub read: bool,
    pub write: bool,
    pub approve: bool,
}

/// A group's matrix
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionMatrixDto {
    pub group: PermissionGroupDto,
    pub rows: Vec<MatrixRowDto>,
}
