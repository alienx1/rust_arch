//! SUPG004 employee search for the account form

use serde::{Deserialize, Serialize};

/// Search for employees that can still get an account
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EmployeeOptionsQuery {
    pub search: Option<String>,
}
