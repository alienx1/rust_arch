//! Tests for scope-aware authorization: which assignments of a user reach a unit.

use application_master::features::su::access_control::{ScopeTarget, grant_applies};
use domain::entity::sea_orm_active_enums::ScopeLevel;
use uuid::Uuid;

fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}

// org 1 > company 2 > branches 10 (A) and 11 (B) > departments 100 (in A), 110 (in B)
fn in_branch_a() -> ScopeTarget {
    ScopeTarget::department(id(1), id(2), id(10), id(100))
}
fn in_branch_b() -> ScopeTarget {
    ScopeTarget::department(id(1), id(2), id(11), id(110))
}

#[test]
fn test_without_a_target_any_granting_assignment_is_enough() {
    assert!(grant_applies(&[Some((ScopeLevel::Branch, id(10)))], None));
    assert!(grant_applies(&[None], None));
    assert!(!grant_applies(&[], None));
}

#[test]
fn test_assignment_applies_only_inside_its_unit() {
    // group "1,2,3" assigned at branch A: allowed in A, refused in B
    let at_a = [Some((ScopeLevel::Branch, id(10)))];
    assert!(grant_applies(&at_a, Some(&in_branch_a())));
    assert!(!grant_applies(&at_a, Some(&in_branch_b())));
}

#[test]
fn test_two_assignments_give_different_rights_per_branch() {
    // program 3 is granted by the group held at A only, program 6 by the group held at B only
    let program_3 = [Some((ScopeLevel::Branch, id(10)))];
    let program_6 = [Some((ScopeLevel::Branch, id(11)))];
    assert!(grant_applies(&program_3, Some(&in_branch_a())));
    assert!(!grant_applies(&program_3, Some(&in_branch_b())));
    assert!(grant_applies(&program_6, Some(&in_branch_b())));
    assert!(!grant_applies(&program_6, Some(&in_branch_a())));
}

#[test]
fn test_one_granting_assignment_among_several_is_enough() {
    let both = [Some((ScopeLevel::Branch, id(10))), Some((ScopeLevel::Branch, id(11)))];
    assert!(grant_applies(&both, Some(&in_branch_a())));
    assert!(grant_applies(&both, Some(&in_branch_b())));
}

#[test]
fn test_a_higher_unit_covers_the_units_below_it() {
    let company = [Some((ScopeLevel::Company, id(2)))];
    assert!(grant_applies(&company, Some(&in_branch_a())));
    assert!(grant_applies(&company, Some(&in_branch_b())));
    let other_company = ScopeTarget::branch(id(1), id(3), id(30));
    assert!(!grant_applies(&company, Some(&other_company)));
}

#[test]
fn test_a_lower_unit_does_not_cover_the_unit_above_it() {
    let branch = [Some((ScopeLevel::Branch, id(10)))];
    assert!(!grant_applies(&branch, Some(&ScopeTarget::company(id(1), id(2)))));
}

#[test]
fn test_an_anywhere_assignment_reaches_every_unit() {
    // a user with no assignment rows holds the primary group at any scope
    assert!(grant_applies(&[None], Some(&in_branch_a())));
    assert!(grant_applies(&[None], Some(&in_branch_b())));
}
