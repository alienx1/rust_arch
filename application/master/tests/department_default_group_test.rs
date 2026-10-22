//! A department's default permission group: what it gives (its group at its branch) and how it
//! is merged into the assignments an admin picked for a new account.
//! Located at `application/master/tests/department_default_group_test.rs`.

use application_master::features::su::department_default_group::{default_assignment, with_department_default};
use domain::entity::db_department;
use domain::entity::sea_orm_active_enums::ScopeLevel;
use uuid::Uuid;

fn department(default_group: Option<Uuid>) -> db_department::Model {
    db_department::Model {
        id: Uuid::now_v7(),
        organization_id: Uuid::now_v7(),
        company_id: Uuid::now_v7(),
        branch_id: Uuid::now_v7(),
        code: "IT".into(),
        active: true,
        default_permission_group_id: default_group,
        change_by: None,
        row_version: 1,
    }
}

#[test]
fn test_the_default_is_the_group_at_the_departments_branch() {
    let group = Uuid::now_v7();
    let d = department(Some(group));
    assert_eq!(default_assignment(&d), Some((group, ScopeLevel::Branch, d.branch_id)), "branch scope, not the department itself");
    assert_eq!(default_assignment(&department(None)), None);
}

#[test]
fn test_the_default_is_added_to_what_was_picked_and_the_pick_stays_first() {
    let (picked, default_group, unit) = (Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
    let d = department(Some(default_group));
    let merged = with_department_default(vec![(picked, ScopeLevel::Company, unit)], default_assignment(&d));
    assert_eq!(merged.len(), 2);
    assert_eq!(merged[0].0, picked, "the first pick is the account's primary group");
    assert_eq!(merged[1], (default_group, ScopeLevel::Branch, d.branch_id));
}

#[test]
fn test_a_group_the_admin_already_gave_is_not_added_again() {
    let group = Uuid::now_v7();
    let d = department(Some(group));
    // Even at another scope: the admin's decision stands
    let merged = with_department_default(vec![(group, ScopeLevel::Company, d.company_id)], default_assignment(&d));
    assert_eq!(merged, vec![(group, ScopeLevel::Company, d.company_id)]);
}

#[test]
fn test_with_nothing_picked_the_default_is_the_only_assignment() {
    let group = Uuid::now_v7();
    let d = department(Some(group));
    assert_eq!(with_department_default(vec![], default_assignment(&d)), vec![(group, ScopeLevel::Branch, d.branch_id)]);
    assert!(with_department_default(vec![], None).is_empty(), "no default, no pick: the caller refuses it");
    let pick = (Uuid::now_v7(), ScopeLevel::Branch, Uuid::now_v7());
    assert_eq!(with_department_default(vec![pick.clone()], None), vec![pick], "a department without a default adds nothing");
}
