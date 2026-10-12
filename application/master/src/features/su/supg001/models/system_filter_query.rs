//! SUPG001 system list filter

use serde::{Deserialize, Serialize};

/// Filter for the system list (`search` matches code and names)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SystemFilterQuery {
    pub search: Option<String>,
    pub active: Option<bool>,
}
