//! SUPG001 update command DTO shared by systems and modules

use serde::{Deserialize, Serialize};

/// Update names / status; the code (and a module's system) is fixed because permissions, routes
/// and the menu reference it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateUnitCommand {
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}
