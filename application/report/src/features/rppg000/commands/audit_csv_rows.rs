//! CSV form of audit events for the IT export: one flat row per event, readable in Excel / any
//! viewer. Pure functions (unit-tested in `application/report/tests/rppg000_test.rs`).
//!
//! - UTF-8 with a byte order mark so Excel shows Thai text correctly; CRLF line ends (RFC 4180).
//! - Times twice: `time_th` (UTC+7, `YYYY-MM-DD HH:MM:SS`, sorts and filters in Excel) and
//!   `time_utc` (RFC 3339 as stored).
//! - Headers / bodies are left out (too large to read); `changes` is a one-line before -> after
//!   rendering of a data change, cut at [`CHANGES_CELL_MAX`] characters.
//! - A text cell that starts with `= + - @` is prefixed with `'` so a spreadsheet never runs it
//!   as a formula (CSV injection from user-controlled paths / names / values).

use application_core::traits::AuditEvent;
use chrono::{DateTime, FixedOffset, Utc};
use serde_json::Value;

pub const BOM: &str = "\u{feff}";
/// Longest `changes` cell; the rest is replaced by `…`
pub const CHANGES_CELL_MAX: usize = 1500;
/// Longest single before / after value inside the `changes` cell
const VALUE_MAX: usize = 80;
const ROW_VERSION: &str = "row_version";

pub const HEADER: [&str; 21] = [
    "time_th",
    "time_utc",
    "kind",
    "user_id",
    "username",
    "email",
    "ip",
    "method",
    "path",
    "query",
    "status",
    "duration_ms",
    "program",
    "action",
    "page",
    "element",
    "label",
    "tables",
    "summary",
    "changes",
    "request_id",
];

/// Header line of the file (without the byte order mark), ending in CRLF
pub fn header_line() -> String {
    let mut line = HEADER.join(",");
    line.push_str("\r\n");
    line
}

/// Quotes a field when it holds a comma, quote or line break, doubling inner quotes
pub fn escape_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Text cell: formula-looking starts are neutralised, then CSV-escaped
fn text_cell(value: Option<&str>) -> String {
    let v = value.unwrap_or("").trim_end();
    if v.starts_with(['=', '+', '-', '@', '\t']) {
        escape_field(&format!("'{v}"))
    } else {
        escape_field(v)
    }
}

fn number_cell<T: ToString>(value: Option<T>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}

fn thai_time(timestamp: &str) -> String {
    let Ok(utc) = DateTime::parse_from_rfc3339(timestamp).map(|d| d.with_timezone(&Utc)) else {
        return String::new();
    };
    let Some(th) = FixedOffset::east_opt(7 * 3600) else { return String::new() };
    utc.with_timezone(&th).format("%Y-%m-%d %H:%M:%S").to_string()
}

fn short(value: &Value) -> String {
    let text = match value {
        Value::Null => "null".to_string(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    let flat = text.replace(['\r', '\n'], " ");
    if flat.chars().count() > VALUE_MAX {
        format!("{}…", flat.chars().take(VALUE_MAX).collect::<String>())
    } else {
        flat
    }
}

fn row_fields(row: Option<&Value>) -> String {
    row.and_then(Value::as_object)
        .map(|o| {
            o.iter()
                .filter(|(k, v)| k.as_str() != ROW_VERSION && !v.is_null())
                .map(|(k, v)| format!("{k}={}", short(v)))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}

/// `table OP row_id: col: before -> after; ...` per changed row, rows separated by ` | `
pub fn changes_text(changes: &str) -> String {
    let Ok(Value::Array(entries)) = serde_json::from_str::<Value>(changes) else {
        return String::new();
    };
    let lines: Vec<String> = entries
        .iter()
        .map(|e| {
            let table = e["table"].as_str().unwrap_or("?");
            let op = e["op"].as_str().unwrap_or("?");
            let id = e["row_id"].as_str().unwrap_or("");
            let body = match op {
                "UPDATE" => e["columns"]
                    .as_array()
                    .map(|cols| {
                        cols.iter()
                            .filter_map(Value::as_str)
                            .map(|c| format!("{c}: {} -> {}", short(&e["old"][c]), short(&e["new"][c])))
                            .collect::<Vec<_>>()
                            .join("; ")
                    })
                    .unwrap_or_default(),
                "DELETE" => row_fields(e.get("old")),
                _ => row_fields(e.get("new")),
            };
            let head = if id.is_empty() { format!("{table} {op}") } else { format!("{table} {op} {id}") };
            if body.is_empty() { head } else { format!("{head}: {body}") }
        })
        .collect();
    let joined = lines.join(" | ");
    if joined.chars().count() > CHANGES_CELL_MAX {
        format!("{}…", joined.chars().take(CHANGES_CELL_MAX).collect::<String>())
    } else {
        joined
    }
}

/// One CSV line (CRLF terminated) for `event`, columns as in [`HEADER`]
pub fn event_line(event: &AuditEvent) -> String {
    let changes = event.changes.as_deref().map(changes_text);
    let cells = [
        thai_time(&event.timestamp),
        escape_field(&event.timestamp),
        text_cell(Some(&event.kind)),
        text_cell(event.user_id.as_deref()),
        text_cell(event.username.as_deref()),
        text_cell(event.email.as_deref()),
        text_cell(event.ip.as_deref()),
        text_cell(event.method.as_deref()),
        text_cell(event.path.as_deref()),
        text_cell(event.query.as_deref()),
        number_cell(event.status),
        number_cell(event.duration_ms),
        text_cell(event.program_code.as_deref()),
        text_cell(event.action.as_deref()),
        text_cell(event.page.as_deref()),
        text_cell(event.element.as_deref()),
        text_cell(event.label.as_deref()),
        text_cell(event.table_name.as_deref()),
        text_cell(event.detail.as_deref()),
        text_cell(changes.as_deref()),
        text_cell(event.request_id.as_deref()),
    ];
    let mut line = cells.join(",");
    line.push_str("\r\n");
    line
}
