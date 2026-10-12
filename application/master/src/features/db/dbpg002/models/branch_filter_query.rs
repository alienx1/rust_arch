//! DBPG002 models: filter for the branch list

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Filter for the branch list
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BranchFilterQuery {
    pub company_id: Option<Uuid>,
    pub search: Option<String>,
    pub active: Option<bool>,
}
