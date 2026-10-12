//! The write -> cache-namespace table: every program that edits cached data must clear what is
//! built from it. Located at `handler/api/master/tests/cache_policy_test.rs`.

use api_master::cache_policy::invalidation_rules;
use application_core::traits::cache_ns;
use axum::http::Method;
use infrastructure::middleware::namespaces_for;

fn cleared(method: Method, path: &str) -> Vec<&'static str> {
    namespaces_for(&invalidation_rules(), &method, path).into_iter().collect()
}

fn assert_clears(path: &str, expected: &[&str]) {
    let got = cleared(Method::PUT, path);
    for ns in expected {
        assert!(got.contains(ns), "a write to {path} must clear `{ns}`, clears {got:?}");
    }
}

#[test]
fn test_permission_writes_clear_permissions_and_menus() {
    for path in ["/api/v1/su/supg003/groups/1/matrix", "/api/v1/su/supg004/users/1", "/api/v1/su/supg005/users/1/suspend"] {
        assert_clears(path, &[cache_ns::ACL, cache_ns::MENU]);
    }
    // Switching an employee off suspends their accounts
    assert_clears("/api/v1/db/dbpg003/employees/1", &[cache_ns::ACL, cache_ns::MENU]);
}

#[test]
fn test_catalogue_writes_clear_programs_menu_and_permissions() {
    for path in ["/api/v1/su/supg001/systems/1", "/api/v1/su/supg002/programs/1"] {
        assert_clears(path, &[cache_ns::PROGRAMS, cache_ns::MENU, cache_ns::ACL, cache_ns::DASHBOARD]);
    }
}

#[test]
fn test_data_writes_clear_their_reads_and_the_dashboard_counters() {
    assert_clears("/api/v1/db/dbpg004/lookups/1", &[cache_ns::REF, cache_ns::DASHBOARD]);
    assert_clears("/api/v1/db/dbpg005/provinces", &[cache_ns::REF, cache_ns::DASHBOARD]);
    assert_clears("/api/v1/db/dbpg006/labels", &[cache_ns::I18N, cache_ns::DASHBOARD]);
    for path in ["/api/v1/db/dbpg000/organizations", "/api/v1/db/dbpg001/companies", "/api/v1/db/dbpg002/branches", "/api/v1/db/dbpg003/employees"] {
        assert_clears(path, &[cache_ns::ORG, cache_ns::DASHBOARD]);
    }
}

#[test]
fn test_both_url_forms_of_a_program_are_covered() {
    assert!(!cleared(Method::POST, "/api/v1/master/db/dbpg005/districts").is_empty());
    assert!(!cleared(Method::POST, "/api/v1/db/dbpg005/districts").is_empty());
}

#[test]
fn test_reads_and_unrelated_writes_clear_nothing() {
    assert!(cleared(Method::GET, "/api/v1/db/dbpg005/provinces").is_empty());
    assert!(cleared(Method::POST, "/api/v1/audit/events").is_empty());
    assert!(cleared(Method::POST, "/api/v1/al/alpg000/exports").is_empty());
    assert!(cleared(Method::POST, "/api/v1/al/alpg001/processes/1/reprocess").is_empty());
}

#[test]
fn test_every_namespace_is_cleared_by_some_write() {
    let all: std::collections::BTreeSet<&str> =
        invalidation_rules().iter().flat_map(|r| r.namespaces.iter().copied()).collect();
    for ns in cache_ns::ALL {
        assert!(all.contains(ns), "namespace `{ns}` is cached but no write ever clears it");
    }
}
