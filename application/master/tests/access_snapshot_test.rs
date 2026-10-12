//! Tests for the permission snapshot that DbAccessControl caches in Valkey: the decisions it makes
//! without a database. Located at `application/master/tests/access_snapshot_test.rs`.

use application_master::features::su::access_control::{AccessSnapshot, Grant, PermissionAction, ScopeTarget};
use domain::entity::sea_orm_active_enums::ScopeLevel;
use uuid::Uuid;

fn grant(program: &str, action: &str, group: Uuid) -> Grant {
    Grant { program: program.into(), action: action.into(), group }
}

fn snapshot(holdings: Vec<(Uuid, Option<(ScopeLevel, Uuid)>)>, grants: Vec<Grant>) -> AccessSnapshot {
    let active_groups = holdings.iter().map(|(g, _)| *g).collect();
    AccessSnapshot { user: Some((Uuid::now_v7(), "somchai".into())), holdings, active_groups, grants }
}

#[test]
fn test_an_unknown_or_switched_off_account_may_do_nothing() {
    let none = AccessSnapshot::default();
    assert!(none.authorize("DBPG004", PermissionAction::Read, None).is_none());
    assert!(none.permissions().is_empty());
}

#[test]
fn test_a_grant_allows_exactly_its_program_and_action() {
    let g = Uuid::now_v7();
    let s = snapshot(vec![(g, None)], vec![grant("DBPG004", "READ", g)]);
    let actor = s.authorize("DBPG004", PermissionAction::Read, None).expect("allowed");
    assert_eq!(actor.username, "somchai");
    assert!(s.authorize("DBPG004", PermissionAction::Write, None).is_none());
    assert!(s.authorize("DBPG005", PermissionAction::Read, None).is_none());
}

#[test]
fn test_actions_match_by_code_ignoring_case() {
    let g = Uuid::now_v7();
    let s = snapshot(vec![(g, None)], vec![grant("SUPG003", "write", g)]);
    assert!(s.authorize("SUPG003", PermissionAction::Write, None).is_some());
}

#[test]
fn test_a_switched_off_group_grants_nothing() {
    let (on, off) = (Uuid::now_v7(), Uuid::now_v7());
    let mut s = snapshot(vec![(on, None), (off, None)], vec![grant("DBPG001", "READ", off)]);
    s.active_groups = vec![on];
    assert!(s.authorize("DBPG001", PermissionAction::Read, None).is_none());
}

#[test]
fn test_a_scoped_grant_only_counts_inside_its_unit() {
    let g = Uuid::now_v7();
    let (org, other_org) = (Uuid::now_v7(), Uuid::now_v7());
    let s = snapshot(vec![(g, Some((ScopeLevel::Organization, org)))], vec![grant("DBPG003", "WRITE", g)]);
    // "may the user do this anywhere?" is yes; for a unit it depends on where the unit is
    assert!(s.authorize("DBPG003", PermissionAction::Write, None).is_some());
    assert!(s.authorize("DBPG003", PermissionAction::Write, Some(&ScopeTarget::organization(org))).is_some());
    assert!(s.authorize("DBPG003", PermissionAction::Write, Some(&ScopeTarget::organization(other_org))).is_none());
}

#[test]
fn test_permissions_union_every_group_by_program() {
    let (a, b) = (Uuid::now_v7(), Uuid::now_v7());
    let s = snapshot(
        vec![(a, None), (b, None)],
        vec![grant("DBPG004", "READ", a), grant("DBPG004", "WRITE", b), grant("SUPG001", "READ", a)],
    );
    let p = s.permissions();
    assert_eq!(p["DBPG004"].iter().copied().collect::<Vec<_>>(), vec!["READ", "WRITE"]);
    assert_eq!(p["SUPG001"].iter().copied().collect::<Vec<_>>(), vec!["READ"]);
}

#[test]
fn test_the_snapshot_survives_the_round_trip_through_valkey() {
    let g = Uuid::now_v7();
    let s = snapshot(vec![(g, Some((ScopeLevel::Branch, Uuid::now_v7())))], vec![grant("DBPG002", "READ", g)]);
    let back: AccessSnapshot = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(back, s);
}
