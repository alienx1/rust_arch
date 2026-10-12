//! DBPG002 models: filter for the department list

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Filter for the department list
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DepartmentFilterQuery {
    pub company_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub search: Option<String>,
    pub active: Option<bool>,
}
