//! DBPG002 models: command to create a branch

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Command DTO to create a branch (its address/contact groups are created with it)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateBranchCommand {
    pub company_id: Uuid,
    pub code: String,
    pub master_lookup_id: Uuid,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}
