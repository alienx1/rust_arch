//! SUPG004 constants: page sizes and the employee picker limit

pub const DEFAULT_PAGE_SIZE: u64 = 25;
pub const MAX_PAGE_SIZE: u64 = 200;
/// Employees offered when linking a new account
pub const EMPLOYEE_OPTIONS_LIMIT: u64 = 25;

/// The group with full access to every program; placing an account in it needs APPROVE on SUPG004
pub const ADMIN_GROUP_CODE: &str = "ADMIN";

pub fn is_admin_group(code: &str) -> bool {
    code.eq_ignore_ascii_case(ADMIN_GROUP_CODE)
}
