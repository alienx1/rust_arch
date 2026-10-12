//! SUPG003 constants: the protected ADMIN group and the permission-administration program

/// The group with full access to every program; its matrix is read-only and it cannot be
/// switched off.
pub const ADMIN_GROUP_CODE: &str = "ADMIN";

/// Program that administers permissions: a group that edits its own matrix must keep READ and
/// WRITE on it.
pub const PERMISSION_PROGRAM_CODE: &str = "SUPG003";

pub fn is_admin_group(code: &str) -> bool {
    code.eq_ignore_ascii_case(ADMIN_GROUP_CODE)
}
