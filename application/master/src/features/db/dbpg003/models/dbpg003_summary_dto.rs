//! DBPG003 models: employee KPI summary

use serde::{Deserialize, Serialize};

/// Employee KPI summary
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dbpg003SummaryDto {
    pub total_employees: u64,
    pub active_employees: u64,
    pub with_user_account: u64,
}
