//! Who sees what: headquarters the organization (or the company / branch it picks), everybody else
//! their own branch plus what their permission groups reach.
//! Run: `cargo test -p application-master --test view_scope_for_test`

use application_core::view_scope::ViewScope;
use application_master::features::su::access_control::{GroupReach, ReachedBranch, Viewer, view_scope_for};
use uuid::Uuid;

fn id() -> Uuid {
    Uuid::now_v7()
}

fn hq() -> Viewer {
    Viewer { is_hq: true, unlimited: true, branches: vec![], departments: vec![], ..Viewer::own_branch(id(), id(), id()) }
}

#[test]
fn test_headquarters_sees_the_whole_organization_unless_it_picks_a_company_or_a_branch() {
    let v = hq();
    assert_eq!(view_scope_for(&v, None, None, None).unwrap(), ViewScope::default());
    let picked = id();
    assert_eq!(view_scope_for(&v, None, Some(picked), None).unwrap(), ViewScope::one_company(picked));
    let (branch, its_company, other_company) = (id(), id(), id());
    assert_eq!(
        view_scope_for(&v, None, Some(other_company), Some((branch, its_company))).unwrap(),
        ViewScope::one_branch(its_company, branch),
        "a branch of another company than the picked one wins: the branch is the more specific choice"
    );
}

#[test]
fn test_an_account_with_a_group_at_the_organization_sees_everything_too() {
    let v = Viewer { unlimited: true, ..Viewer::own_branch(id(), id(), id()) };
    assert_eq!(view_scope_for(&v, None, None, None).unwrap(), ViewScope::default());
}

#[test]
fn test_an_ordinary_account_sees_its_own_branch_whatever_it_picks() {
    let v = Viewer::own_branch(id(), id(), id());
    let own = ViewScope::one_branch(v.company_id, v.branch_id);
    assert_eq!(view_scope_for(&v, None, None, None).unwrap(), own);
    assert_eq!(view_scope_for(&v, None, Some(id()), Some((id(), id()))).unwrap(), own, "a pick outside its reach cannot widen the view");
}

fn multi() -> (Viewer, Uuid, Uuid, Uuid) {
    // Works in `own`; its groups reach `second` (same company) and `abroad` (another company)
    let (org, company, other_company) = (id(), id(), id());
    let (own, second, abroad) = (id(), id(), id());
    let viewer = Viewer {
        branches: vec![
            ReachedBranch { branch_id: own, company_id: company, whole: true },
            ReachedBranch { branch_id: second, company_id: company, whole: true },
            ReachedBranch { branch_id: abroad, company_id: other_company, whole: true },
        ],
        ..Viewer::own_branch(org, company, own)
    };
    (viewer, own, second, abroad)
}

#[test]
fn test_groups_that_reach_more_branches_widen_the_view_to_all_of_them() {
    let (v, own, second, abroad) = multi();
    let scope = view_scope_for(&v, None, None, None).unwrap();
    let mut branches = scope.branches.clone().unwrap();
    branches.sort();
    let mut expected = vec![own, second, abroad];
    expected.sort();
    assert_eq!(branches, expected);
    assert_eq!(scope.companies.as_ref().unwrap().len(), 2, "both companies show");
    assert!(scope.partial_branches.is_empty() && scope.departments.is_empty());
}

#[test]
fn test_a_pick_narrows_to_what_is_reached_and_is_ignored_otherwise() {
    let (v, _own, second, abroad) = multi();
    let company_of_second = v.branches.iter().find(|r| r.branch_id == second).unwrap().company_id;

    let one = view_scope_for(&v, None, None, Some((abroad, Uuid::nil()))).unwrap();
    assert_eq!(one.branches, Some(vec![abroad]), "a reached branch can be picked");
    assert_eq!(one.single_branch(), Some(abroad));

    let company = view_scope_for(&v, None, Some(company_of_second), None).unwrap();
    assert_eq!(company.branches.as_ref().unwrap().len(), 2, "the company pick keeps the branches of that company");

    let unreached = view_scope_for(&v, None, None, Some((id(), id()))).unwrap();
    assert_eq!(unreached.branches.as_ref().unwrap().len(), 3, "an unreached branch is no choice: nothing narrows");
}

#[test]
fn test_a_group_at_a_department_shows_that_department_in_its_branch_but_not_the_branch() {
    let (org, company, own, remote, dept) = (id(), id(), id(), id(), id());
    let viewer = Viewer {
        branches: vec![
            ReachedBranch { branch_id: own, company_id: company, whole: true },
            ReachedBranch { branch_id: remote, company_id: company, whole: false },
        ],
        departments: vec![(dept, remote)],
        ..Viewer::own_branch(org, company, own)
    };
    let scope = view_scope_for(&viewer, None, None, None).unwrap();
    assert_eq!(scope.branches, Some(vec![own]), "only the own branch is visible as a whole");
    assert_eq!(scope.partial_branches, vec![remote], "the remote branch row shows");
    assert_eq!(scope.departments, vec![dept], "with just the granted department");
    assert_eq!(scope.branch_rows().unwrap().len(), 2);
    assert_eq!(scope.single_branch(), None);

    // Picking the partly reached branch leaves only that department
    let picked = view_scope_for(&viewer, None, None, Some((remote, Uuid::nil()))).unwrap();
    assert_eq!(picked.branches, Some(vec![]));
    assert_eq!(picked.partial_branches, vec![remote]);
    assert_eq!(picked.departments, vec![dept]);
}

/// Works in `home` (group `plain` reaches it: no dashboard) and is also given `other` through group
/// `dashboard` (which grants the dashboard): `(viewer, plain, dashboard, home, other)`
fn two_groups() -> (Viewer, Uuid, Uuid, Uuid, Uuid) {
    let (org, company) = (id(), id());
    let (plain, dashboard, home, other) = (id(), id(), id(), id());
    let reach = |group_id, branch_id| GroupReach {
        group_id,
        unlimited: false,
        branches: vec![ReachedBranch { branch_id, company_id: company, whole: true }],
        departments: vec![],
    };
    let viewer = Viewer {
        branches: vec![
            ReachedBranch { branch_id: home, company_id: company, whole: true },
            ReachedBranch { branch_id: other, company_id: company, whole: true },
        ],
        reaches: vec![reach(plain, home), reach(dashboard, other)],
        ..Viewer::own_branch(org, company, home)
    };
    (viewer, plain, dashboard, home, other)
}

#[test]
fn test_a_screen_shows_data_only_from_branches_where_the_account_holds_its_permission() {
    let (v, plain, dashboard, home, other) = two_groups();
    // Without a screen in mind both branches count
    assert_eq!(view_scope_for(&v, None, None, None).unwrap().branches.unwrap().len(), 2);
    // The dashboard is granted by `dashboard` only: just where that group reaches
    let scope = view_scope_for(&v, Some(&[dashboard]), None, None).unwrap();
    assert_eq!(scope.branches, Some(vec![other]), "the home branch is not part of the dashboard");
    // A screen both groups grant shows both
    let both = view_scope_for(&v, Some(&[plain, dashboard]), None, None).unwrap();
    assert_eq!(both.branches.unwrap().len(), 2);
    let _ = home;
}

#[test]
fn test_picking_a_branch_where_the_screen_is_not_granted_is_refused_not_ignored() {
    let (v, _plain, dashboard, home, other) = two_groups();
    assert!(view_scope_for(&v, Some(&[dashboard]), None, Some((home, Uuid::nil()))).is_none(), "main branch: no dashboard there");
    assert_eq!(
        view_scope_for(&v, Some(&[dashboard]), None, Some((other, Uuid::nil()))).unwrap().single_branch(),
        Some(other)
    );
    // A company outside the reach is refused as well; without a screen it is only ignored
    assert!(view_scope_for(&v, Some(&[dashboard]), Some(id()), None).is_none());
    assert!(view_scope_for(&v, None, Some(id()), None).is_some());
}

#[test]
fn test_headquarters_sees_the_organization_in_general_but_each_screen_follows_its_own_groups() {
    let (org, company, home, other, dashboard) = (id(), id(), id(), id(), id());
    let hq = Viewer {
        is_hq: true,
        unlimited: true,
        branches: vec![],
        reaches: vec![GroupReach {
            group_id: dashboard,
            unlimited: false,
            departments: vec![],
            branches: vec![ReachedBranch { branch_id: other, company_id: company, whole: true }],
        }],
        ..Viewer::own_branch(org, company, home)
    };
    assert_eq!(view_scope_for(&hq, None, None, None).unwrap(), ViewScope::default(), "no screen: the organization");
    let dash = view_scope_for(&hq, Some(&[dashboard]), None, None).unwrap();
    assert_eq!(dash.branches, Some(vec![other]), "the dashboard is held in one branch only, headquarters or not");
    assert!(view_scope_for(&hq, Some(&[dashboard]), None, Some((home, company))).is_none());

    // A group at the whole organization keeps the headquarters' overview on every screen
    let all = Viewer {
        reaches: vec![GroupReach { group_id: dashboard, unlimited: true, departments: vec![], branches: vec![] }],
        ..hq.clone()
    };
    assert_eq!(view_scope_for(&all, Some(&[dashboard]), None, None).unwrap(), ViewScope::default());
}
