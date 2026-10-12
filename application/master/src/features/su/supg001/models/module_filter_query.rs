//! SUPG001 module list filter

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Filter for the module list (`search` matches code and names)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModuleFilterQuery {
    pub system_id: Option<Uuid>,
    pub search: Option<String>,
    pub active: Option<bool>,
}
