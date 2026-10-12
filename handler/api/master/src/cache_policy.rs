//! Which writes clear which Valkey read caches (see `infrastructure::middleware::cache_invalidation`).
//!
//! The namespaces are those of `application_core::traits::cache_ns`; a write to a program clears
//! every cached read built from the data that program edits. When a new cached read is added to an
//! executor, make sure the programs that write its data are listed here.

use application_core::traits::cache_ns::{ACL, DASHBOARD, I18N, MENU, ORG, PROGRAMS, REF};
use infrastructure::middleware::InvalidationRule;

/// Path (below `/api/v1`, with or without the `/master` prefix) -> namespaces to clear
pub fn invalidation_rules() -> Vec<InvalidationRule> {
    vec![
        // Organization structure and employees: option lists and counters. Switching an employee
        // off suspends the accounts linked to it, so permissions and menus change as well.
        InvalidationRule { path: "/db/dbpg000", namespaces: &[ORG, DASHBOARD] },
        InvalidationRule { path: "/db/dbpg001", namespaces: &[ORG, DASHBOARD] },
        InvalidationRule { path: "/db/dbpg002", namespaces: &[ORG, DASHBOARD] },
        InvalidationRule { path: "/db/dbpg003", namespaces: &[ORG, DASHBOARD, ACL, MENU] },
        // Reference data (lookups, geography, types, prefixes)
        InvalidationRule { path: "/db/dbpg004", namespaces: &[REF, ORG, DASHBOARD] },
        InvalidationRule { path: "/db/dbpg005", namespaces: &[REF, DASHBOARD] },
        // Labels: dictionaries and API error templates
        InvalidationRule { path: "/db/dbpg006", namespaces: &[I18N, DASHBOARD] },
        // Security administration: systems, modules, programs, groups and the matrix, users, sessions
        InvalidationRule { path: "/su/supg001", namespaces: &[PROGRAMS, MENU, ACL, DASHBOARD] },
        InvalidationRule { path: "/su/supg002", namespaces: &[PROGRAMS, MENU, ACL, DASHBOARD] },
        InvalidationRule { path: "/su/supg003", namespaces: &[ACL, MENU, ORG] },
        InvalidationRule { path: "/su/supg004", namespaces: &[ACL, MENU, ORG, DASHBOARD] },
        InvalidationRule { path: "/su/supg005", namespaces: &[ACL, MENU, DASHBOARD] },
    ]
}
