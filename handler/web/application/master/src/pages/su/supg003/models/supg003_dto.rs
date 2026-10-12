//! SUPG003 DTOs: permission groups, the permission matrix and write payloads

use serde::{Deserialize, Serialize};
use crate::pages::db::dbpg000::models::{deserialize_id, deserialize_opt_id, serialize_id};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionGroupDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub code: String,
    pub active: bool,
    pub user_count: u64,
    pub readable_programs: u64,
    pub is_protected: bool,
    pub row_version: i64,
}

/// One program row of the matrix
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatrixRowDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub program_id: String,
    pub system_code: String,
    pub module_code: String,
    pub program_code: String,
    pub name_th: String,
    pub name_en: String,
    #[serde(default, deserialize_with = "deserialize_opt_id")]
    pub parent_id: Option<String>,
    pub depth: u32,
    pub is_folder: bool,
    pub program_active: bool,
    pub read: bool,
    pub write: bool,
    pub approve: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionMatrixDto {
    pub group: PermissionGroupDto,
    pub rows: Vec<MatrixRowDto>,
}

/// Values of the group form
#[derive(Clone, Debug, Default)]
pub struct GroupFormInput {
    pub code: String,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateGroupInput {
    pub code: String,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateGroupInput {
    pub active: bool,
    pub row_version: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrantInput {
    #[serde(serialize_with = "serialize_id")]
    pub program_id: String,
    pub read: bool,
    pub write: bool,
    pub approve: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveMatrixInput {
    pub grants: Vec<GrantInput>,
}
