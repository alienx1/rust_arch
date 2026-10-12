//! Program code → URL path table (mirrors `db_program.path`, lower-cased).
//!
//! The only place web code should build links to program pages. A new program adds its
//! constant here, its `<Route>` in `master_routes.rs`, and its `pages/<program_code>/` module.

/// `SUPG001` — Systems & Modules (list)
pub const SUPG001: &str = "/su/supg001";
/// `SUPG002` — Menus & Programs (list)
pub const SUPG002: &str = "/su/supg002";
/// `SUPG002` — Program form (create / edit)
pub const SUPG002_PROGRAM: &str = "/su/supg002/program";
/// `SUPG003` — Permission Matrix (group list)
pub const SUPG003: &str = "/su/supg003";
/// `SUPG003` — Group form (create / edit)
pub const SUPG003_GROUP: &str = "/su/supg003/group";
/// `SUPG004` — User Accounts (list)
pub const SUPG004: &str = "/su/supg004";
/// `SUPG004` — Account form (create / edit)
pub const SUPG004_USER: &str = "/su/supg004/user";
/// `ALPG000` — Audit Log (list)
pub const ALPG000: &str = "/al/alpg000";
/// `ALPG000` — Audit event detail (`/al/alpg000/log/{id}`)
pub const ALPG000_LOG: &str = "/al/alpg000/log";
/// `NTPG000` — Notifications (list)
pub const NTPG000: &str = "/nt/ntpg000";
/// Dev-only UI components demo (not a catalogue program)
pub const DEMO: &str = "/demo";
/// `SYPG000` — Dashboard
pub const SYPG000: &str = "/";
/// `DBPG000` — Organizations & Companies (list)
pub const DBPG000: &str = "/db/dbpg000";
/// `DBPG002` — Branches & Departments (list)
pub const DBPG002: &str = "/db/dbpg002";
/// `DBPG002` — Branch form (create / edit)
pub const DBPG002_BRANCH: &str = "/db/dbpg002/branch";
/// `DBPG002` — Department form (create / edit)
pub const DBPG002_DEPARTMENT: &str = "/db/dbpg002/department";
/// `DBPG005` — Geography (list)
pub const DBPG005: &str = "/db/dbpg005";
/// `DBPG004` — Reference Data (list)
pub const DBPG004: &str = "/db/dbpg004";
/// `DBPG004` — Reference value form (create / edit)
pub const DBPG004_LOOKUP: &str = "/db/dbpg004/lookup";
/// `DBPG006` — i18n Labels (list)
pub const DBPG006: &str = "/db/dbpg006";
/// `DBPG006` — Label form (create / edit)
pub const DBPG006_LABEL: &str = "/db/dbpg006/label";
/// `DBPG003` — Employees (list)
pub const DBPG003: &str = "/db/dbpg003";
/// `DBPG003` — Employee form (create / edit)
pub const DBPG003_EMPLOYEE: &str = "/db/dbpg003/employee";
/// `DBPG001` — Organization detail form (create / edit)
pub const DBPG001_ORG: &str = "/db/dbpg001/org";
/// `DBPG001` — Company detail form (create / edit)
pub const DBPG001_COMPANY: &str = "/db/dbpg001/company";

/// Organization form path: `/db/dbpg001/org` (create) or `/db/dbpg001/org/{id}` (edit).
pub fn dbpg001_org_path(id: Option<&str>) -> String {
    match id {
        Some(id) => format!("{DBPG001_ORG}/{id}"),
        None => DBPG001_ORG.to_string(),
    }
}

/// Company form path: `/db/dbpg001/company` (create) or `/db/dbpg001/company/{id}` (edit).
pub fn dbpg001_company_path(id: Option<&str>) -> String {
    match id {
        Some(id) => format!("{DBPG001_COMPANY}/{id}"),
        None => DBPG001_COMPANY.to_string(),
    }
}

/// Branch form path: `/db/dbpg002/branch` (create) or `/db/dbpg002/branch/{id}` (edit).
pub fn dbpg002_branch_path(id: Option<&str>) -> String {
    match id {
        Some(id) => format!("{DBPG002_BRANCH}/{id}"),
        None => DBPG002_BRANCH.to_string(),
    }
}

/// Department form path: `/db/dbpg002/department` (create) or `/db/dbpg002/department/{id}` (edit).
pub fn dbpg002_department_path(id: Option<&str>) -> String {
    match id {
        Some(id) => format!("{DBPG002_DEPARTMENT}/{id}"),
        None => DBPG002_DEPARTMENT.to_string(),
    }
}

/// Employee form path: `/db/dbpg003/employee` (create) or `/db/dbpg003/employee/{id}` (edit).
pub fn dbpg003_employee_path(id: Option<&str>) -> String {
    match id {
        Some(id) => format!("{DBPG003_EMPLOYEE}/{id}"),
        None => DBPG003_EMPLOYEE.to_string(),
    }
}

/// Reference value form path: `/db/dbpg004/lookup` (create) or `/db/dbpg004/lookup/{id}` (edit).
pub fn dbpg004_lookup_path(id: Option<&str>) -> String {
    match id {
        Some(id) => format!("{DBPG004_LOOKUP}/{id}"),
        None => DBPG004_LOOKUP.to_string(),
    }
}

/// Percent-encodes `value` for a URL query component (everything but unreserved characters).
pub fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Label form path: `/db/dbpg006/label` (create, optionally with a pre-selected category) or
/// `/db/dbpg006/label?category=…&key=…` (edit: the entry is identified by both query parameters,
/// since keys contain dots and may contain any character).
pub fn dbpg006_label_path(category: Option<&str>, key: Option<&str>) -> String {
    match (category, key) {
        (Some(c), Some(k)) => {
            format!("{DBPG006_LABEL}?category={}&key={}", percent_encode(c), percent_encode(k))
        }
        (Some(c), None) => format!("{DBPG006_LABEL}?category={}", percent_encode(c)),
        _ => DBPG006_LABEL.to_string(),
    }
}

/// Geography form path: `/db/dbpg005/{level}` (create) or `/db/dbpg005/{level}/{id}` (edit), where
/// the level slug is `province`, `district`, `sub-district` or `postal-code`.
pub fn dbpg005_form_path(level: crate::pages::db::dbpg005::models::GeoLevel, id: Option<&str>) -> String {
    match id {
        Some(id) => format!("{DBPG005}/{}/{id}", level.slug()),
        None => format!("{DBPG005}/{}", level.slug()),
    }
}

/// System / module form path: `/su/supg001/{kind}` (create) or `/su/supg001/{kind}/{id}` (edit),
/// where the kind slug is `system` or `module`.
pub fn supg001_form_path(kind: crate::pages::su::supg001::models::UnitKind, id: Option<&str>) -> String {
    match id {
        Some(id) => format!("{SUPG001}/{}/{id}", kind.slug()),
        None => format!("{SUPG001}/{}", kind.slug()),
    }
}

/// Program form path: `/su/supg002/program` (create) or `/su/supg002/program/{id}` (edit).
pub fn supg002_program_path(id: Option<&str>) -> String {
    match id {
        Some(id) => format!("{SUPG002_PROGRAM}/{id}"),
        None => SUPG002_PROGRAM.to_string(),
    }
}

/// Group form path: `/su/supg003/group` (create) or `/su/supg003/group/{id}` (edit).
pub fn supg003_group_path(id: Option<&str>) -> String {
    match id {
        Some(id) => format!("{SUPG003_GROUP}/{id}"),
        None => SUPG003_GROUP.to_string(),
    }
}

/// Matrix editor path of a group: `/su/supg003/group/{id}/matrix`.
pub fn supg003_matrix_path(id: &str) -> String {
    format!("{SUPG003_GROUP}/{id}/matrix")
}

/// Account form path: `/su/supg004/user` (create) or `/su/supg004/user/{id}` (edit).
pub fn supg004_user_path(id: Option<&str>) -> String {
    match id {
        Some(id) => format!("{SUPG004_USER}/{id}"),
        None => SUPG004_USER.to_string(),
    }
}

/// Audit event detail path: `/al/alpg000/log/{id}` (the id is percent-encoded).
pub fn alpg000_log_path(id: &str) -> String {
    format!("{ALPG000_LOG}/{}", percent_encode(id))
}
