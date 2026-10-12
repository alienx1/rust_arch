//! Tests for SUPG002 (menu tree): visibility rules of `build_menu_tree`.
//! Located at `application/master/tests/supg002_test.rs`.

use application_master::features::su::supg002::build_menu_tree;
use domain::entity::db_program;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

fn program(code: &str, parent: Option<Uuid>, path: Option<&str>, sort_order: i32) -> db_program::Model {
    db_program::Model {
        id: Uuid::now_v7(),
        system_id: Uuid::now_v7(),
        module_id: Uuid::now_v7(),
        program_id: parent,
        code: code.to_string(),
        path: path.map(str::to_string),
        icon: Some("ApartmentOutlined".to_string()),
        sort_order,
        active: true,
        change_by: None,
        row_version: 1,
    }
}

fn names(programs: &[&db_program::Model]) -> HashMap<Uuid, (String, String)> {
    programs.iter().map(|p| (p.id, (format!("ชื่อ {}", p.code), format!("Name {}", p.code)))).collect()
}

#[test]
fn test_permitted_leaf_restores_its_folder_and_hides_the_rest() {
    let folder = program("DBPG000", None, Some("/db/dbpg000"), 2);
    let allowed = program("DBPG002", Some(folder.id), Some("/db/dbpg002"), 2);
    let denied = program("DBPG003", Some(folder.id), Some("/db/dbpg003"), 3);
    let other_folder = program("SUPG000", None, None, 3);
    let other_leaf = program("SUPG005", Some(other_folder.id), Some("/su/supg005"), 5);

    let permitted: HashSet<Uuid> = [allowed.id].into_iter().collect();
    let all = vec![folder.clone(), allowed.clone(), denied.clone(), other_folder.clone(), other_leaf.clone()];
    let tree = build_menu_tree(&all, &permitted, &names(&[&folder, &allowed, &denied, &other_folder, &other_leaf]));

    assert_eq!(tree.len(), 1, "only the folder above the permitted leaf is shown");
    assert_eq!(tree[0].code, "DBPG000");
    assert_eq!(tree[0].children.len(), 1);
    assert_eq!(tree[0].children[0].code, "DBPG002");
    assert_eq!(tree[0].children[0].label_th, "ชื่อ DBPG002");
}

#[test]
fn test_siblings_sort_by_sort_order_then_code_and_blank_path_becomes_none() {
    let root_b = program("BBB", None, Some("/b"), 1);
    let root_a = program("AAA", None, Some("  "), 1);
    let root_first = program("ZZZ", None, Some("/z"), 0);
    let permitted: HashSet<Uuid> = [root_b.id, root_a.id, root_first.id].into_iter().collect();
    let all = vec![root_b.clone(), root_a.clone(), root_first.clone()];

    let tree = build_menu_tree(&all, &permitted, &HashMap::new());
    let codes: Vec<_> = tree.iter().map(|n| n.code.as_str()).collect();
    assert_eq!(codes, ["ZZZ", "AAA", "BBB"]);
    assert_eq!(tree[1].path, None, "a blank path is not navigable");
    assert_eq!(tree[0].label_th, "ZZZ", "falls back to the code when there is no label");
}

#[test]
fn test_switching_a_folder_off_hides_its_whole_subtree() {
    // The folder is absent from `programs` (inactive, or in an inactive system/module): its
    // permitted children must not resurface as roots.
    let folder = program("DBPG000", None, None, 1);
    let child = program("DBPG002", Some(folder.id), Some("/db/dbpg002"), 1);
    let grandchild = program("DBPG002A", Some(child.id), Some("/db/dbpg002a"), 1);
    let permitted: HashSet<Uuid> = [child.id, grandchild.id].into_iter().collect();

    let tree = build_menu_tree(&[child, grandchild], &permitted, &HashMap::new());
    assert!(tree.is_empty());
}

#[test]
fn test_no_permissions_gives_an_empty_tree() {
    let folder = program("DBPG000", None, None, 1);
    let leaf = program("DBPG002", Some(folder.id), Some("/db/dbpg002"), 1);
    let tree = build_menu_tree(&[folder, leaf], &HashSet::new(), &HashMap::new());
    assert!(tree.is_empty());
}

#[test]
fn test_program_code_rules() {
    use application_master::features::su::supg002::{Supg002Error, normalize_program_code};
    assert_eq!(normalize_program_code(" dbpg006 ").unwrap(), "DBPG006");
    assert_eq!(normalize_program_code("alpg000_group").unwrap(), "ALPG000_GROUP");
    for bad in ["", "AB", "HAS SPACE", "A-B-C", "ไทยไทย", &"X".repeat(31)] {
        assert!(
            matches!(normalize_program_code(bad), Err(Supg002Error::Validation(_))),
            "{bad:?} must be rejected"
        );
    }
}

#[test]
fn test_program_path_rules() {
    use application_master::features::su::supg002::{Supg002Error, normalize_program_path};
    let n = |s: &str| normalize_program_path(&Some(s.to_string()));

    assert_eq!(n("/DB/DBPG006").unwrap().as_deref(), Some("/db/dbpg006"), "stored lower case");
    assert_eq!(n("/").unwrap().as_deref(), Some("/"));
    assert_eq!(n("  ").unwrap(), None, "blank means a folder without a page");
    assert_eq!(normalize_program_path(&None).unwrap(), None);
    for bad in ["db/dbpg006", "/db//x", "/db/x/", "/db/ex ample", "/db/ไทย", "/db/x?y=1"] {
        assert!(
            matches!(n(bad), Err(Supg002Error::Validation(_))),
            "{bad:?} must be rejected"
        );
    }
}

#[test]
fn test_menu_admin_pages_are_protected_from_being_switched_off() {
    use application_master::features::su::supg002::is_protected_program;
    for code in ["SUPG001", "supg002", "SUPG003"] {
        assert!(is_protected_program(code), "{code}");
    }
    assert!(!is_protected_program("DBPG003") && !is_protected_program("SUPG005"));
}
