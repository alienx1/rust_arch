//! Filter + paging query for the geography lists

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Filter + paging for the geography lists (`search` matches code and names)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GeoFilterQuery {
    pub province_id: Option<Uuid>,
    pub district_id: Option<Uuid>,
    pub search: Option<String>,
    pub active: Option<bool>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}
