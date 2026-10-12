//! KPI summary DTO

use serde::{Deserialize, Serialize};

/// Row counts per level
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dbpg005SummaryDto {
    pub provinces: u64,
    pub districts: u64,
    pub sub_districts: u64,
    pub postal_codes: u64,
}
