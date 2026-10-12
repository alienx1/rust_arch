//! Tests for SUPG004 (User Accounts): input rules checked before anything reaches the identity
//! provider. Located at `application/master/tests/supg004_test.rs`.

use application_master::features::su::supg004::{
    AssignmentCommand, Supg004Error, is_valid_email, normalize_assignments, normalize_username,
};
use domain::entity::sea_orm_active_enums::ScopeLevel;
use uuid::Uuid;

fn pick(group: Uuid, level: &str, unit: Uuid) -> AssignmentCommand {
    AssignmentCommand { permission_group_id: group, scope_level: level.to_string(), scope_id: unit }
}

#[test]
fn test_at_least_one_assignment_is_required() {
    assert!(matches!(normalize_assignments(&[]), Err(Supg004Error::Validation(_))));
}

#[test]
fn test_same_group_and_unit_collapse_and_order_is_kept() {
    let (a, b, u1, u2) = (Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
    let out = normalize_assignments(&[
        pick(a, "Branch", u1),
        pick(b, "branch", u2),
        pick(a, "branch", u1),
        pick(a, "company", u2),
    ])
    .unwrap();
    assert_eq!(
        out,
        vec![(a, ScopeLevel::Branch, u1), (b, ScopeLevel::Branch, u2), (a, ScopeLevel::Company, u2)]
    );
}

#[test]
fn test_unknown_scope_level_is_rejected() {
    let r = normalize_assignments(&[pick(Uuid::now_v7(), "region", Uuid::now_v7())]);
    assert!(matches!(r, Err(Supg004Error::Validation(_))));
}

#[test]
fn test_username_follows_the_identity_schema_and_is_stored_lower_case() {
    assert_eq!(normalize_username(" Somchai.J ").unwrap(), "somchai.j");
    assert_eq!(normalize_username("a_b-c.9").unwrap(), "a_b-c.9");
    for bad in ["", "ab", "has space", "a@b.c", "ไทยไทย", "semi;colon", &"x".repeat(65)] {
        assert!(
            matches!(normalize_username(bad), Err(Supg004Error::Validation(_))),
            "{bad:?} must be rejected"
        );
    }
}

#[test]
fn test_email_check_accepts_plain_addresses_only() {
    for ok in ["admin@arch.local", "a.b+c@corp.example.com", " user@x.io "] {
        assert!(is_valid_email(ok), "{ok:?}");
    }
    for bad in ["", "no-at-sign", "@no-local.com", "user@", "user@nodot", "user@.com", "user@com.", "a b@x.io", "a@b@c.io"] {
        assert!(!is_valid_email(bad), "{bad:?}");
    }
}
