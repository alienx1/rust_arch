//! SUPG002 program list filter

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Filter + paging for the program list (`search` matches code, path and names)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProgramFilterQuery {
    pub system_id: Option<Uuid>,
    pub module_id: Option<Uuid>,
    pub search: Option<String>,
    pub active: Option<bool>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}
