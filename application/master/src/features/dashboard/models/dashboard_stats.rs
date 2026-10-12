use serde::{Deserialize, Serialize};

/// Summary KPI metrics across Master Data and Identity subsystems.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
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

/// Module summary descriptor with localized labels.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleItem {
    pub code: String,
    pub label_th: String,
    pub label_en: String,
    pub system_code: String,
    pub active: bool,
}

/// Program summary descriptor for quick navigation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgramItem {
    pub code: String,
    pub label_th: String,
    pub label_en: String,
    pub active: bool,
}

/// Unified Dashboard summary response containing real metrics from database.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DashboardSummary {
    pub kpis: DashboardKpis,
    pub modules: Vec<ModuleItem>,
    pub recent_programs: Vec<ProgramItem>,
    pub region: String,
    pub architecture: String,
}
