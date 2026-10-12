//! Which requests the audit trail records and what program / action they belong to.
//! Pure functions of method and path (unit-tested in `tests/audit_trail_test.rs`).

/// Segments of `path` after `/api/v1` and the optional `/master` service prefix;
/// `None` when the path is not an API path
fn api_segments(path: &str) -> Option<Vec<&str>> {
    let mut segs = path.split('/').filter(|s| !s.is_empty());
    if segs.next()? != "api" || segs.next()? != "v1" {
        return None;
    }
    let mut rest: Vec<&str> = segs.collect();
    if rest.first() == Some(&"master") {
        rest.remove(0);
    }
    Some(rest)
}

/// `true` for catalogue program codes in a path segment (`dbpg004`, `supg001`, `alpg000`)
fn is_program_segment(seg: &str) -> bool {
    let b = seg.as_bytes();
    b.len() == 7
        && b[..2].iter().all(u8::is_ascii_alphabetic)
        && b[2..4].eq_ignore_ascii_case(b"pg")
        && b[4..].iter().all(u8::is_ascii_digit)
}

/// Whether a request is written to the audit log: API paths only (`/api/v1/**`), without the
/// client event ingestion (`/audit/**`), the audit page's own reads (`/al/alpg000` GETs; they
/// would log themselves forever), dictionary fetches (`/i18n`, `/localization`), health probes
/// and CORS preflights.
pub fn should_audit(method: &str, path: &str) -> bool {
    if method.eq_ignore_ascii_case("OPTIONS") {
        return false;
    }
    let Some(segs) = api_segments(path) else { return false };
    let Some(first) = segs.first() else { return false };
    if matches!(*first, "audit" | "i18n" | "localization" | "health" | "livez" | "readyz") {
        return false;
    }
    let is_read = method.eq_ignore_ascii_case("GET") || method.eq_ignore_ascii_case("HEAD");
    !(is_read && segs.iter().any(|s| s.eq_ignore_ascii_case("alpg000")))
}

/// `(program_code, action)` of a request. Program: the `xxpgNNN` path segment upper-cased,
/// `dashboard` -> `SYPG000`, hooks -> none. Action: from the method (`GET` VIEW, `POST` CREATE,
/// `PUT`/`PATCH` UPDATE, `DELETE` DELETE) refined by the path (`login-guard` LOGIN,
/// `recovery-link` RECOVERY_LINK, `PUT .../matrix` UPDATE_MATRIX, `suspend`, `reactivate`,
/// `revoke*` / `DELETE .../sessions...` REVOKE).
pub fn derive_audit_target(method: &str, path: &str) -> (Option<String>, String) {
    let segs = api_segments(path).unwrap_or_else(|| path.split('/').filter(|s| !s.is_empty()).collect());
    let program = segs
        .iter()
        .find(|s| is_program_segment(s))
        .map(|s| s.to_ascii_uppercase())
        .or_else(|| (segs.first() == Some(&"dashboard")).then(|| "SYPG000".to_string()));

    let method = method.to_ascii_uppercase();
    let last = segs.last().copied().unwrap_or("");
    let action = if segs.first() == Some(&"hooks") && last == "login-guard" {
        "LOGIN".to_string()
    } else if last == "recovery-link" {
        "RECOVERY_LINK".to_string()
    } else if method == "PUT" && last == "matrix" {
        "UPDATE_MATRIX".to_string()
    } else if last == "suspend" {
        "SUSPEND".to_string()
    } else if last == "reactivate" {
        "REACTIVATE".to_string()
    } else if segs.iter().any(|s| s.starts_with("revoke")) || (method == "DELETE" && segs.contains(&"sessions")) {
        "REVOKE".to_string()
    } else {
        match method.as_str() {
            "GET" | "HEAD" => "VIEW".to_string(),
            "POST" => "CREATE".to_string(),
            "PUT" | "PATCH" => "UPDATE".to_string(),
            "DELETE" => "DELETE".to_string(),
            other => other.to_string(),
        }
    };
    (program, action)
}
