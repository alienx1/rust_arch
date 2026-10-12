//! ALPG000 search filter rules

use application_core::traits::{AuditSearch, audit_kind};
use chrono::DateTime;

use crate::features::al::alpg000::errors::Alpg000Error;
use crate::features::al::alpg000::models::{
    AuditLogFilterQuery, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE, MAX_TEXT_LEN,
};

fn clean(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn rfc3339(name: &str, value: Option<String>) -> Result<Option<String>, Alpg000Error> {
    let Some(v) = clean(value) else { return Ok(None) };
    DateTime::parse_from_rfc3339(&v)
        .map_err(|_| Alpg000Error::Validation(format!("`{name}` must be an RFC 3339 timestamp")))?;
    Ok(Some(v))
}

/// Turns the query string into the store filter: page >= 1, limit default 25 / max 200,
/// timestamps RFC 3339, known `kind` (a single kind, or `ui` for page views + clicks + actions), `status_class` 2..=5, `text` at most 200 characters;
/// blank values are dropped.
pub fn normalize_search(q: AuditLogFilterQuery) -> Result<AuditSearch, Alpg000Error> {
    let kind = clean(q.kind).map(|k| k.to_lowercase());
    if let Some(k) = &kind {
        let known = [
            audit_kind::PAGE_VIEW,
            audit_kind::CLICK,
            audit_kind::ACTION,
            audit_kind::API,
            audit_kind::DATA_CHANGE,
            audit_kind::UI,
            audit_kind::EVENT_REPORT_REQUEST,
            audit_kind::EVENT_NOTIFICATION,
        ];
        if !known.contains(&k.as_str()) {
            return Err(Alpg000Error::Validation(format!("Unknown kind `{k}`")));
        }
    }
    if let Some(c) = q.status_class {
        if !(2..=5).contains(&c) {
            return Err(Alpg000Error::Validation("`status_class` must be 2, 3, 4 or 5".to_string()));
        }
    }
    let text = clean(q.text);
    if text.as_ref().is_some_and(|t| t.chars().count() > MAX_TEXT_LEN) {
        return Err(Alpg000Error::Validation(format!("`text` is limited to {MAX_TEXT_LEN} characters")));
    }
    Ok(AuditSearch {
        from: rfc3339("from", q.from)?,
        to: rfc3339("to", q.to)?,
        kind,
        user: clean(q.user),
        program_code: clean(q.program_code).map(|p| p.to_uppercase()),
        method: clean(q.method).map(|m| m.to_uppercase()),
        status: q.status,
        status_class: q.status_class,
        path: clean(q.path),
        text,
        page: q.page.unwrap_or(1).max(1),
        limit: q.limit.filter(|l| *l > 0).unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_PAGE_SIZE),
    })
}
