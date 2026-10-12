//! SUPG002 create-program command DTO

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Command DTO to create a program. System and module are fixed afterwards: permission rows carry
/// them too.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateProgramCommand {
    pub system_id: Uuid,
    pub module_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub code: String,
    pub path: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}
