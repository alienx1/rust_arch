//! DBPG003 models: one page of employees

use super::employee_dto::EmployeeDto;
use serde::{Deserialize, Serialize};

/// One page of employees
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmployeePageDto {
    pub items: Vec<EmployeeDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}
