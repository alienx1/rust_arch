//! Pure builders between the audit port and Quickwit: query string, time range, paging and the
//! JSON document form of an [`AuditEvent`]. No I/O here so everything is unit-tested.
//!
//! Path filter: the mapping has `path` (default tokenizer: split on non-alphanumerics, lower
//! case, positions recorded) and `path_raw` (raw, the whole path). A "substring of the path" is
//! implemented as a token match on `path`: `dbpg004` finds every path under that program, and
//! `dbpg004/users` becomes the phrase `path:"dbpg004 users"`. A single token is a prefix match
//! (`supg*`). Arbitrary character substrings are not supported (they would need a wildcard scan).

use application_core::traits::{AuditError, AuditEvent, AuditSearch, audit_kind};
use chrono::{DateTime, Utc};
use serde_json::{Map, Value, json};

pub const DEFAULT_PAGE_LIMIT: u64 = 50;
pub const MAX_PAGE_LIMIT: u64 = 200;

/// Quotes a value for an exact (raw tokenizer) field match; `\` and `"` are escaped so user input
/// cannot leave the quotes and inject query syntax.
pub fn quote_term(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '\\' | '"' => {
                out.push('\\');
                out.push(c);
            }
            // Control characters carry no meaning in an identifier
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Alphanumeric words of `input` (Unicode aware), lower-cased; everything else separates words
pub(super) fn words(input: &str) -> Vec<String> {
    input
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

pub(super) fn non_empty(v: &Option<String>) -> Option<&str> {
    v.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

/// Query string for `filter` (`*` when nothing narrows the result). Time range and paging are not
/// part of the query: see [`audit_time_range`] and [`page_window`]. A `status` wins over a
/// `status_class`; a class outside 1..=5 is ignored.
pub fn build_audit_query(filter: &AuditSearch) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(v) = non_empty(&filter.kind) {
        if v == audit_kind::UI {
            // The group of everything the web app reports
            let kinds: Vec<String> =
                audit_kind::UI_KINDS.iter().map(|k| format!("kind:{}", quote_term(k))).collect();
            parts.push(format!("({})", kinds.join(" OR ")));
        } else {
            parts.push(format!("kind:{}", quote_term(v)));
        }
    }
    if let Some(v) = non_empty(&filter.program_code) {
        parts.push(format!("program_code:{}", quote_term(v)));
    }
    if let Some(v) = non_empty(&filter.method) {
        parts.push(format!("method:{}", quote_term(&v.to_ascii_uppercase())));
    }
    if let Some(status) = filter.status {
        parts.push(format!("status:{status}"));
    } else if let Some(class) = filter.status_class.filter(|c| (1..=5).contains(c)) {
        parts.push(format!("status:[{} TO {}]", u32::from(class) * 100, u32::from(class) * 100 + 99));
    }
    if let Some(v) = non_empty(&filter.user) {
        let q = quote_term(v);
        parts.push(format!("(user_id:{q} OR username:{q} OR email:{q})"));
    }
    if let Some(v) = non_empty(&filter.path) {
        let tokens = words(v);
        match tokens.as_slice() {
            [] => {}
            [one] => parts.push(format!("path:{one}*")),
            many => parts.push(format!("path:\"{}\"", many.join(" "))),
        }
    }
    if let Some(v) = non_empty(&filter.text) {
        // Every word must occur somewhere in the default search fields; words are alphanumeric
        // only and always quoted, so AND / OR / NOT and field syntax are plain terms.
        for w in words(v) {
            parts.push(format!("\"{w}\""));
        }
    }

    if parts.is_empty() {
        "*".to_string()
    } else {
        parts.join(" AND ")
    }
}

/// `start_timestamp` (inclusive) / `end_timestamp` (exclusive) in unix seconds
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AuditTimeRange {
    pub start: Option<i64>,
    pub end: Option<i64>,
}

fn parse_rfc3339(label: &str, raw: &str) -> Result<DateTime<Utc>, AuditError> {
    DateTime::parse_from_rfc3339(raw.trim())
        .map(|d| d.with_timezone(&Utc))
        .map_err(|e| AuditError::InvalidQuery(format!("`{label}` is not an RFC 3339 time: {e}")))
}

/// `from` / `to` -> Quickwit time range. `from` rounds down and `to` rounds up to a whole second
/// (`to` is exclusive in Quickwit too, so a sub-second `to` must not cut its own second).
pub fn audit_time_range(filter: &AuditSearch) -> Result<AuditTimeRange, AuditError> {
    let start = non_empty(&filter.from).map(|v| parse_rfc3339("from", v)).transpose()?.map(|d| d.timestamp());
    let end = non_empty(&filter.to)
        .map(|v| parse_rfc3339("to", v))
        .transpose()?
        .map(|d| d.timestamp() + i64::from(d.timestamp_subsec_nanos() > 0));
    Ok(AuditTimeRange { start, end })
}

/// `(max_hits, start_offset)` for the 1-based `page` / `limit` (limit clamped to 1..=200,
/// 0 means the default of 50)
pub fn page_window(filter: &AuditSearch) -> (u64, u64) {
    let limit = if filter.limit == 0 { DEFAULT_PAGE_LIMIT } else { filter.limit.min(MAX_PAGE_LIMIT) };
    let page = filter.page.max(1);
    (limit, (page - 1).saturating_mul(limit))
}

/// Document ingested into Quickwit: the event as JSON without nulls, plus `path_raw` (copy of
/// `path`, the raw-tokenized twin of the tokenized `path` field)
pub fn event_to_doc(event: &AuditEvent) -> Value {
    let mut doc = match serde_json::to_value(event) {
        Ok(Value::Object(m)) => m,
        _ => Map::new(),
    };
    doc.retain(|_, v| !v.is_null());
    if let Some(path) = event.path.as_ref().filter(|p| !p.is_empty()) {
        doc.insert("path_raw".to_string(), Value::String(path.clone()));
    }
    Value::Object(doc)
}

/// Inverse of [`event_to_doc`] for a search hit (`path_raw` is dropped)
pub fn doc_to_event(mut doc: Value) -> Result<AuditEvent, AuditError> {
    if let Value::Object(m) = &mut doc {
        m.remove("path_raw");
    }
    serde_json::from_value(doc).map_err(|e| AuditError::Unavailable(format!("unreadable audit document: {e}")))
}

/// Events per scroll page of an export
pub const EXPORT_PAGE_SIZE: u64 = 5000;
/// How long Quickwit keeps a scroll open between two pages
pub const EXPORT_SCROLL_KEEP_ALIVE: &str = "2m";

fn unix_to_rfc3339(secs: i64) -> String {
    DateTime::<Utc>::from_timestamp(secs, 0).map(|d| d.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)).unwrap_or_default()
}

/// Body of the first Elasticsearch-compatible `_search?scroll=` call of an export: the same query
/// and time range as the list, newest first, [`EXPORT_PAGE_SIZE`] events per page
pub fn export_search_body(filter: &AuditSearch) -> Result<Value, AuditError> {
    let range = audit_time_range(filter)?;
    let mut bounds = Map::new();
    if let Some(start) = range.start {
        bounds.insert("gte".into(), Value::String(unix_to_rfc3339(start)));
    }
    if let Some(end) = range.end {
        bounds.insert("lt".into(), Value::String(unix_to_rfc3339(end)));
    }
    let mut bool_query = Map::new();
    bool_query.insert("must".into(), json!([{ "query_string": { "query": build_audit_query(filter) } }]));
    if !bounds.is_empty() {
        bool_query.insert("filter".into(), json!([{ "range": { "timestamp": Value::Object(bounds) } }]));
    }
    Ok(json!({
        "query": { "bool": Value::Object(bool_query) },
        "size": EXPORT_PAGE_SIZE,
        "sort": [{ "timestamp": { "order": "desc" } }],
    }))
}
