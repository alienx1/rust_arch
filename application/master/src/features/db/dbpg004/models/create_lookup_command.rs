//! DBPG004 create-lookup command DTO

use serde::{Deserialize, Serialize};

/// Command DTO to create a lookup value. A new `category` name creates the category.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateLookupCommand {
    pub category: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}
