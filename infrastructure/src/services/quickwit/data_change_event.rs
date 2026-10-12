//! Pure conversion of `al_data_change` outbox rows into [`AuditEvent`]s (`kind=data_change`):
//! the rows of one API request become ONE event, secrets masked, noise left out. No I/O, so it is
//! unit-tested.

use application_core::traits::{AuditEvent, audit_kind};
use serde_json::{Map, Value, json};

/// One row of `al_data_change`, every column as text
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataChangeRow {
    pub id: String,
    /// RFC 3339, UTC
    pub changed_at: String,
    pub txid: String,
    pub table_name: String,
    /// `INSERT` / `UPDATE` / `DELETE`
    pub op: String,
    pub row_id: Option<String>,
    pub actor_id: String,
    pub actor_name: Option<String>,
    pub request_id: Option<String>,
    /// JSON text
    pub old_data: Option<String>,
    pub new_data: Option<String>,
}

const MASK: &str = "***";
const SECRET_WORDS: [&str; 6] = ["password", "secret", "token", "credential", "api_key", "cookie"];
/// Bookkeeping column: bumped by every write, says nothing about the data
const VERSION_COLUMN: &str = "row_version";

fn is_secret(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    SECRET_WORDS.iter().any(|w| key.contains(w))
}

fn mask(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, v) in map.iter_mut() {
                if is_secret(key) && !v.is_null() {
                    *v = Value::String(MASK.to_string());
                } else {
                    mask(v);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(mask),
        _ => {}
    }
}

fn parse(text: &Option<String>) -> Option<Value> {
    text.as_deref().and_then(|t| serde_json::from_str(t).ok())
}

/// Columns whose value differs between the two row images, in column order, without the
/// bookkeeping `row_version`
fn changed_columns(old: &Value, new: &Value) -> Vec<String> {
    let (Some(old), Some(new)) = (old.as_object(), new.as_object()) else { return Vec::new() };
    new.iter()
        .filter(|(k, v)| k.as_str() != VERSION_COLUMN && old.get(*k) != Some(*v))
        .map(|(k, _)| k.clone())
        .collect()
}

/// An UPDATE that only moved `row_version` (e.g. a label upsert of unchanged text): not a change
fn is_noise(row: &DataChangeRow, old: &Option<Value>, new: &Option<Value>) -> bool {
    row.op == "UPDATE" && matches!((old, new), (Some(o), Some(n)) if changed_columns(o, n).is_empty())
}

/// Groups rows that belong together: the same API request, else the same transaction. Order of
/// first appearance is kept.
pub fn group_changes(rows: Vec<DataChangeRow>) -> Vec<Vec<DataChangeRow>> {
    let mut groups: Vec<(String, Vec<DataChangeRow>)> = Vec::new();
    for row in rows {
        let key = row.request_id.clone().unwrap_or_else(|| format!("tx:{}", row.txid));
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, g)) => g.push(row),
            None => groups.push((key, vec![row])),
        }
    }
    groups.into_iter().map(|(_, g)| g).collect()
}

/// One event for the rows of one request; `None` when nothing but noise is left
pub fn changes_to_event(rows: &[DataChangeRow]) -> Option<AuditEvent> {
    let mut entries: Vec<Value> = Vec::new();
    let mut tables: Vec<String> = Vec::new();
    let mut ops: Vec<String> = Vec::new();
    // (table, op) -> count, in order of first appearance, for the summary line
    let mut tally: Vec<(String, String, usize)> = Vec::new();

    let mut ordered: Vec<&DataChangeRow> = rows.iter().collect();
    ordered.sort_by(|a, b| a.changed_at.cmp(&b.changed_at).then(a.id.cmp(&b.id)));
    let first = *ordered.first()?;

    for row in ordered.iter().copied() {
        let (old, new) = (parse(&row.old_data), parse(&row.new_data));
        if is_noise(row, &old, &new) {
            continue;
        }
        let columns = match (&old, &new) {
            (Some(o), Some(n)) => changed_columns(o, n),
            _ => Vec::new(),
        };
        let render = |v: Option<Value>| {
            v.map(|mut v| {
                mask(&mut v);
                v
            })
            .unwrap_or(Value::Null)
        };
        let mut entry = Map::new();
        entry.insert("table".into(), json!(row.table_name));
        entry.insert("op".into(), json!(row.op));
        entry.insert("row_id".into(), json!(row.row_id));
        entry.insert("columns".into(), json!(columns));
        entry.insert("old".into(), render(old));
        entry.insert("new".into(), render(new));
        entries.push(Value::Object(entry));

        if !tables.contains(&row.table_name) {
            tables.push(row.table_name.clone());
        }
        if !ops.contains(&row.op) {
            ops.push(row.op.clone());
        }
        match tally.iter_mut().find(|(t, o, _)| *t == row.table_name && *o == row.op) {
            Some((_, _, n)) => *n += 1,
            None => tally.push((row.table_name.clone(), row.op.clone(), 1)),
        }
    }
    if entries.is_empty() {
        return None;
    }

    let detail = tally
        .iter()
        .map(|(t, o, n)| if *n > 1 { format!("{t} {o} ×{n}") } else { format!("{t} {o}") })
        .collect::<Vec<_>>()
        .join(", ");
    Some(AuditEvent {
        event_id: first.id.clone(),
        timestamp: first.changed_at.clone(),
        kind: audit_kind::DATA_CHANGE.to_string(),
        user_id: Some(first.actor_id.clone()),
        username: first.actor_name.clone(),
        action: Some(if let [only] = ops.as_slice() { only.clone() } else { "CHANGE".to_string() }),
        detail: Some(detail),
        table_name: Some(tables.join(", ")),
        changes: Some(Value::Array(entries).to_string()),
        request_id: first.request_id.clone(),
        ..Default::default()
    })
}
