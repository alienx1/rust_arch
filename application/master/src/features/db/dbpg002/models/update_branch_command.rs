//! DBPG002 models: command to update a branch

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Command DTO to update a branch (the owning company cannot change)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateBranchCommand {
    pub code: String,
    pub master_lookup_id: Uuid,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}
