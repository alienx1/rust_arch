//! SUPG004 employee option DTO

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An active employee without an account
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmployeeOptionDto {
    pub id: Uuid,
    pub code: String,
    pub full_name: String,
    pub department_name: String,
    pub company_name: String,
}
