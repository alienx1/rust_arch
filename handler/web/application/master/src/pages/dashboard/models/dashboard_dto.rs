//! Dashboard Data Transfer Objects (DTOs)

use serde::{Deserialize, Serialize};

/// Aggregated dashboard statistics and operational metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DashboardSummary {
    pub kpis: DashboardKpis,
    pub modules: Vec<ModuleItem>,
    pub recent_programs: Vec<ProgramItem>,
    pub region: String,
    pub architecture: String,
}

/// Real-time entity counts from PostgreSQL database.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DashboardKpis {
    pub companies: u64,
    pub branches: u64,
    pub organizations: u64,
    pub departments: u64,
    pub employees: u64,
    pub provinces: u64,
    pub districts: u64,
    pub sub_districts: u64,
    pub labels: u64,
    pub languages: u64,
    pub systems: u64,
    pub modules: u64,
    pub programs: u64,
    pub users: u64,
}

/// Registered module info.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleItem {
    pub code: String,
    pub label_th: String,
    pub label_en: String,
    pub system_code: String,
    pub active: bool,
}

/// Registered program / screen info.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgramItem {
    pub code: String,
    pub label_th: String,
    #[serde(default)]
    pub label_en: String,
    pub active: bool,
}
