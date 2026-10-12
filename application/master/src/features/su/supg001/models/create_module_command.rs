//! SUPG001 create-module command DTO

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateModuleCommand {
    pub system_id: Uuid,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}
