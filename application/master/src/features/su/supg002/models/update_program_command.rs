//! SUPG002 update-program command DTO

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Command DTO to update a program: place, route, icon, order, status and names (code, system and
/// module are fixed).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateProgramCommand {
    pub parent_id: Option<Uuid>,
    pub path: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}
