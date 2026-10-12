//! SUPG001 protected system / module codes

/// System / module codes that cannot be switched off: the security administration itself lives
/// there, and switching it off would hide the pages needed to switch it back on.
pub const PROTECTED_SYSTEM_CODES: &[&str] = &["SU"];
pub const PROTECTED_MODULE_CODES: &[&str] = &["SUMT"];

pub fn is_protected_system(code: &str) -> bool {
    PROTECTED_SYSTEM_CODES.iter().any(|c| c.eq_ignore_ascii_case(code))
}

pub fn is_protected_module(code: &str) -> bool {
    PROTECTED_MODULE_CODES.iter().any(|c| c.eq_ignore_ascii_case(code))
}
