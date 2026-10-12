//! SUPG002 protected program codes and paging limits

/// Programs whose menu entry cannot be switched off: the pages that administer the menu and the
/// permissions themselves (switching them off would hide the way back).
pub const PROTECTED_PROGRAM_CODES: &[&str] = &["SUPG001", "SUPG002", "SUPG003"];

pub fn is_protected_program(code: &str) -> bool {
    PROTECTED_PROGRAM_CODES.iter().any(|c| c.eq_ignore_ascii_case(code))
}

pub const DEFAULT_PAGE_SIZE: u64 = 25;
pub const MAX_PAGE_SIZE: u64 = 200;
