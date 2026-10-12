//! SUPG001 create-system command DTO

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateSystemCommand {
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}
