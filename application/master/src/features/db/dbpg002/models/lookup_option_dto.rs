//! DBPG002 models: company and branch type options for selectors

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Company option for the company filter / parent selector
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyOptionDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}

/// Permission group for the department's default-group selector
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionGroupOptionDto {
    pub id: Uuid,
    pub code: String,
}

/// Branch type (`branch_type` master lookup: HEAD / BRANCH)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchTypeLookupDto {
    pub id: Uuid,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}
