//! DBPG004 update-lookup command DTO

use serde::{Deserialize, Serialize};

/// Command DTO to update a lookup value. Category and code are fixed once created: other tables
/// and application code reference rows by them.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateLookupCommand {
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}
