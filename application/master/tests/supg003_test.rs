//! Tests for SUPG003 (Permission Matrix): the pure rules behind saving a matrix.
//! Located at `application/master/tests/supg003_test.rs`.

use application_master::features::su::supg003::{
    ChangePlan, CreatePermissionGroupCommand, ExistingGrant, MatrixAction, SaveMatrixCommand, Supg003Error, UpdatePermissionGroupCommand, is_admin_group, keeps_own_access,
    normalize_group_code, order_programs, plan_changes,
};
use domain::entity::db_program;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

fn grant(program: Uuid, action: MatrixAction, active: bool) -> ExistingGrant {
    ExistingGrant { row_id: Uuid::now_v7(), program_id: program, action, active }
}

fn want(items: &[(Uuid, MatrixAction)]) -> HashSet<(Uuid, MatrixAction)> {
    items.iter().copied().collect()
}

#[test]
fn test_group_code_rules_and_admin_detection() {
    assert_eq!(normalize_group_code(" hr_manager ").unwrap(), "HR_MANAGER");
    for bad in ["", "A", "HAS SPACE", "A-B", "ไทย", &"X".repeat(51)] {
        assert!(
            matches!(normalize_group_code(bad), Err(Supg003Error::Validation(_))),
            "{bad:?} must be rejected"
        );
    }
    assert!(is_admin_group("ADMIN") && is_admin_group("admin"));
    assert!(!is_admin_group("ADMINS"));
}

#[test]
fn test_group_and_matrix_dtos_carry_no_scope() {
    // A group is WHAT (programs x actions); scopes live on user assignments (SUPG004)
    let create: CreatePermissionGroupCommand = serde_json::from_str(r#"{"code":"HR","active":true}"#).unwrap();
    assert_eq!((create.code.as_str(), create.active), ("HR", true));
    let update: UpdatePermissionGroupCommand = serde_json::from_str(r#"{"active":false,"row_version":3}"#).unwrap();
    assert_eq!((update.active, update.row_version), (false, 3));
    let save: SaveMatrixCommand = serde_json::from_str(r#"{"grants":[],"row_version":1}"#).unwrap();
    assert!(save.grants.is_empty());
}

#[test]
fn test_duplicate_code_error_and_matrix_passes_through_plan() {
    // Codes are compared after normalisation, so " hr " and "HR" collide as DuplicateCode on create
    assert_eq!(normalize_group_code(" hr ").unwrap(), normalize_group_code("HR").unwrap());
    let err = Supg003Error::DuplicateCode("HR".to_string());
    assert!(err.to_string().contains("HR"));

    // A saved matrix passes through unchanged: the desired set equals the stored one -> no writes;
    // a new READ+WRITE grant is inserted as-is
    let p = Uuid::now_v7();
    let plan = plan_changes(&[], &want(&[(p, MatrixAction::Read), (p, MatrixAction::Write)]));
    assert_eq!(plan.insert.len(), 2);
    assert!(plan.activate.is_empty() && plan.deactivate.is_empty());
}

#[test]
fn test_unchanged_matrix_plans_nothing() {
    let p = Uuid::now_v7();
    let existing = vec![grant(p, MatrixAction::Read, true), grant(p, MatrixAction::Write, true)];
    let plan = plan_changes(&existing, &want(&[(p, MatrixAction::Read), (p, MatrixAction::Write)]));
    assert_eq!(plan, ChangePlan::default());
}

#[test]
fn test_plan_inserts_activates_and_deactivates() {
    let (a, b, c) = (Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
    let off = grant(a, MatrixAction::Read, false); // exists but off -> wanted: switch on
    let on = grant(b, MatrixAction::Read, true); // active -> no longer wanted: switch off
    let keep = grant(b, MatrixAction::Write, true); // active and still wanted: untouched
    let existing = vec![off.clone(), on.clone(), keep];

    let plan = plan_changes(
        &existing,
        &want(&[
            (a, MatrixAction::Read),
            (b, MatrixAction::Write),
            (c, MatrixAction::Read), // no row yet -> insert
            (c, MatrixAction::Approve),
        ]),
    );
    assert_eq!(plan.activate, vec![off.row_id]);
    assert_eq!(plan.deactivate, vec![on.row_id]);
    assert_eq!(plan.insert.len(), 2);
    assert!(plan.insert.iter().all(|(p, _)| *p == c));
}

#[test]
fn test_duplicate_lookup_rows_of_one_grant_are_handled_together() {
    let p = Uuid::now_v7();
    // The permission_type lookup repeats a code, so one grant can have two rows
    let first = grant(p, MatrixAction::Read, true);
    let second = grant(p, MatrixAction::Read, false);

    // Still wanted: an active row already keeps it
    assert_eq!(
        plan_changes(&[first.clone(), second.clone()], &want(&[(p, MatrixAction::Read)])),
        ChangePlan::default()
    );
    // Removed: every active row goes off
    let removed = plan_changes(&[first.clone(), second], &want(&[]));
    assert_eq!(removed.deactivate, vec![first.row_id]);
    assert!(removed.activate.is_empty() && removed.insert.is_empty());
}

#[test]
fn test_own_group_must_keep_read_and_write_on_the_permission_page() {
    let page = Uuid::now_v7();
    let other = Uuid::now_v7();
    assert!(keeps_own_access(&want(&[(page, MatrixAction::Read), (page, MatrixAction::Write)]), page));
    assert!(!keeps_own_access(&want(&[(page, MatrixAction::Read)]), page), "WRITE dropped");
    assert!(!keeps_own_access(&want(&[(page, MatrixAction::Write)]), page), "READ dropped");
    assert!(!keeps_own_access(&want(&[(other, MatrixAction::Read), (other, MatrixAction::Write)]), page));
}

fn program(code: &str, parent: Option<Uuid>, sort_order: i32, system: Uuid, module: Uuid) -> db_program::Model {
    db_program::Model {
        id: Uuid::now_v7(),
        system_id: system,
        module_id: module,
        program_id: parent,
        code: code.to_string(),
        path: None,
        icon: None,
        sort_order,
        active: true,
        change_by: None,
        row_version: 1,
    }
}

#[test]
fn test_matrix_rows_follow_the_tree_with_depth() {
    let (sys_su, sys_db) = (Uuid::now_v7(), Uuid::now_v7());
    let (mod_su, mod_db) = (Uuid::now_v7(), Uuid::now_v7());
    let systems: HashMap<Uuid, String> = [(sys_su, "SU".to_string()), (sys_db, "DB".to_string())].into();
    let modules: HashMap<Uuid, String> = [(mod_su, "SUMT".to_string()), (mod_db, "DBMT".to_string())].into();

    let db_folder = program("DBPG000", None, 2, sys_db, mod_db);
    let db_child_b = program("DBPG003", Some(db_folder.id), 3, sys_db, mod_db);
    let db_child_a = program("DBPG002", Some(db_folder.id), 2, sys_db, mod_db);
    let su_folder = program("SUPG000", None, 3, sys_su, mod_su);
    let su_child = program("SUPG001", Some(su_folder.id), 1, sys_su, mod_su);

    let ordered = order_programs(
        &[su_child, db_child_b, su_folder, db_child_a, db_folder],
        &systems,
        &modules,
    );
    let flat: Vec<(&str, u32)> = ordered.iter().map(|(p, d)| (p.code.as_str(), *d)).collect();
    // DB system before SU (by code), each folder before its children, children by sort order
    assert_eq!(
        flat,
        [("DBPG000", 0), ("DBPG002", 1), ("DBPG003", 1), ("SUPG000", 0), ("SUPG001", 1)]
    );
}
