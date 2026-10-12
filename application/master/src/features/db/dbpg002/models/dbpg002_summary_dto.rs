//! DBPG002 models: KPI summary of branches and departments

use serde::{Deserialize, Serialize};

/// High-level branch and department statistics
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dbpg002SummaryDto {
    pub total_branches: u64,
    pub active_branches: u64,
    pub total_departments: u64,
    pub active_departments: u64,
}
