//! DBPG000 company list filter

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Filter query parameters for searching companies
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CompanyFilterQuery {
    pub organization_id: Option<Uuid>,
    pub search: Option<String>,
    pub active: Option<bool>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}
