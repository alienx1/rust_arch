//! DBPG004 constants: page sizes and system-protected categories

pub const DEFAULT_PAGE_SIZE: u64 = 25;
pub const MAX_PAGE_SIZE: u64 = 200;

/// Categories whose rows the application logic depends on by code (`READ`, `WRITE`, `APPROVE`):
/// their rows cannot be created or deactivated here; labels stay editable.
pub const PROTECTED_CATEGORIES: &[&str] = &["permission_type"];

pub fn is_protected_category(category: &str) -> bool {
    PROTECTED_CATEGORIES
        .iter()
        .any(|c| c.eq_ignore_ascii_case(category))
}
