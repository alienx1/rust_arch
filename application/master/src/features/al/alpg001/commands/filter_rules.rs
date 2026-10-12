//! ALPG001 filter validation

use chrono::{DateTime, Utc};

use crate::features::al::alpg001::errors::Alpg001Error;
use crate::features::al::alpg001::models::{
    DEFAULT_LIMIT, MAX_LIMIT, PROCESS_KINDS, ProcessFilterQuery, ProcessSearch, process_status,
};

fn clean(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn instant(name: &str, v: Option<String>) -> Result<Option<DateTime<Utc>>, Alpg001Error> {
    clean(v)
        .map(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|t| t.with_timezone(&Utc))
                .map_err(|_| Alpg001Error::Validation(format!("`{name}` must be an RFC 3339 timestamp")))
        })
        .transpose()
}

/// Query string -> store filter: known `status` / `kind` (lower-cased), RFC 3339 times, page >= 1,
/// limit default 25 / max 200; blank values are dropped.
pub fn normalize_process_filter(q: ProcessFilterQuery) -> Result<ProcessSearch, Alpg001Error> {
    let status = clean(q.status).map(|s| s.to_lowercase());
    if let Some(s) = &status {
        if !process_status::ALL.contains(&s.as_str()) {
            return Err(Alpg001Error::Validation(format!("Unknown status `{s}`")));
        }
    }
    let kind = clean(q.kind).map(|k| k.to_lowercase());
    if let Some(k) = &kind {
        if !PROCESS_KINDS.contains(&k.as_str()) {
            return Err(Alpg001Error::Validation(format!("Unknown process kind `{k}`")));
        }
    }
    let limit = match q.limit {
        None | Some(0) => DEFAULT_LIMIT,
        Some(n) => n.min(MAX_LIMIT),
    };
    Ok(ProcessSearch {
        status,
        kind,
        user: clean(q.user),
        from: instant("from", q.from)?,
        to: instant("to", q.to)?,
        page: q.page.unwrap_or(1).max(1),
        limit,
    })
}
