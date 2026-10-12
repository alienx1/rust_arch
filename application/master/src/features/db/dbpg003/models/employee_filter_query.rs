//! DBPG003 models: filter and paging for the employee list

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Filter + paging for the employee list (`search` matches code, names and nickname)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EmployeeFilterQuery {
    pub company_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub search: Option<String>,
    pub active: Option<bool>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}
