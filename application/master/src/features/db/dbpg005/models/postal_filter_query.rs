//! Filter + paging query for the postal code list

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Filter + paging for postal codes (`search` matches the code and sub-district / district names)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PostalFilterQuery {
    pub province_id: Option<Uuid>,
    pub district_id: Option<Uuid>,
    pub sub_district_id: Option<Uuid>,
    pub search: Option<String>,
    pub active: Option<bool>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}
