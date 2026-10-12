//! Tests for SUPG001 (Systems & Modules): code rules and protected units.
//! Located at `application/master/tests/supg001_test.rs`.

use application_master::features::su::supg001::{
    Supg001Error, is_protected_module, is_protected_system, normalize_unit_code,
};

#[test]
fn test_unit_codes_are_two_to_ten_upper_case_alphanumerics() {
    assert_eq!(normalize_unit_code(" su ").unwrap(), "SU");
    assert_eq!(normalize_unit_code("acmt2").unwrap(), "ACMT2");
    for bad in ["", "A", "TOO_LONG_CODE", "HAS SPACE", "ไทย", "A-B"] {
        assert!(
            matches!(normalize_unit_code(bad), Err(Supg001Error::Validation(_))),
            "{bad:?} must be rejected"
        );
    }
}

#[test]
fn test_security_system_and_module_are_protected() {
    assert!(is_protected_system("SU") && is_protected_system("su"));
    assert!(is_protected_module("SUMT"));
    assert!(!is_protected_system("DB") && !is_protected_module("DBMT"));
}
