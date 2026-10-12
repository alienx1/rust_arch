//! SUPG004 user account DTOs: an account and one page of accounts

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::UserAssignmentDto;

/// A user account with the employee and group behind it
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserAccountDto {
    pub id: Uuid,
    pub username: String,
    pub employee_id: Uuid,
    pub employee_code: String,
    pub full_name: String,
    pub department_name: String,
    pub company_name: String,
    pub group_id: Uuid,
    pub group_code: String,
    /// Groups held at scope units; empty for a legacy account without assignment rows
    pub assignments: Vec<UserAssignmentDto>,
    pub active: bool,
    /// RFC 3339
    pub created_time: String,
    pub row_version: i64,
}

/// One page of accounts
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserPageDto {
    pub items: Vec<UserAccountDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}
