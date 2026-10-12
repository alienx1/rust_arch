//! SUPG003 group list filter

use serde::{Deserialize, Serialize};

/// Filter for the group list (`search` matches the code)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GroupFilterQuery {
    pub search: Option<String>,
    pub active: Option<bool>,
}
