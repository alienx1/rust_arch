//! Moves `al_data_change` rows (written by the `al_capture_change` trigger) to the audit log.
//!
//! One loop per API instance: every `POLL_INTERVAL` it takes up to `BATCH` rows with
//! `DELETE ... RETURNING` (`FOR UPDATE SKIP LOCKED`, so several instances share the work without
//! doubling events) and groups them per API request into one event each and hands those to [`AuditLogPort::record`]. `record` is fire-and-forget, so
//! like every audit event a change is lost if the store is down for longer than its queue holds.

use super::data_change_event::{DataChangeRow, changes_to_event, group_changes};
use application_core::traits::AuditLogPort;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, QueryResult, Statement};
use std::sync::Arc;
use std::time::Duration;

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const BATCH: u32 = 500;

fn take_batch_sql() -> String {
    format!(
        "WITH batch AS (SELECT \"id\" FROM \"al_data_change\" ORDER BY \"changed_at\", \"id\" LIMIT {BATCH} FOR UPDATE SKIP LOCKED) \
         DELETE FROM \"al_data_change\" d USING batch b WHERE d.\"id\" = b.\"id\" \
         RETURNING d.\"id\"::text AS id, \
                   to_char(d.\"changed_at\" AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"') AS changed_at, \
                   d.\"txid\"::text AS txid, d.\"table_name\" AS table_name, d.\"op\" AS op, d.\"row_id\" AS row_id, d.\"actor_id\" AS actor_id, \
                   d.\"actor_name\" AS actor_name, d.\"request_id\" AS request_id, \
                   d.\"old_data\"::text AS old_data, d.\"new_data\"::text AS new_data"
    )
}

fn text(row: &QueryResult, col: &str) -> Option<String> {
    row.try_get::<Option<String>>("", col).ok().flatten()
}

fn to_row(row: &QueryResult) -> Option<DataChangeRow> {
    Some(DataChangeRow {
        id: text(row, "id")?,
        changed_at: text(row, "changed_at")?,
        txid: text(row, "txid")?,
        table_name: text(row, "table_name")?,
        op: text(row, "op")?,
        row_id: text(row, "row_id"),
        actor_id: text(row, "actor_id")?,
        actor_name: text(row, "actor_name"),
        request_id: text(row, "request_id"),
        old_data: text(row, "old_data"),
        new_data: text(row, "new_data"),
    })
}

/// Ships one batch; returns how many outbox rows it took
pub async fn ship_once(db: &DatabaseConnection, log: &dyn AuditLogPort) -> Result<usize, sea_orm::DbErr> {
    let rows = db.query_all_raw(Statement::from_string(DbBackend::Postgres, take_batch_sql())).await?;
    let rows: Vec<DataChangeRow> = rows.iter().filter_map(to_row).collect();
    let n = rows.len();
    // One event per API request, however many rows it touched
    let events: Vec<_> = group_changes(rows).iter().filter_map(|g| changes_to_event(g)).collect();
    if !events.is_empty() {
        log.record(events);
    }
    Ok(n)
}

/// Starts the shipping loop on the current Tokio runtime
pub fn spawn_data_change_shipper(db: DatabaseConnection, log: Arc<dyn AuditLogPort>) {
    tokio::spawn(async move {
        let mut warned = false;
        loop {
            match ship_once(&db, log.as_ref()).await {
                Ok(n) if n as u32 >= BATCH => continue, // more waiting: no pause
                Ok(_) => warned = false,
                Err(e) => {
                    // e.g. the table is missing on a database loaded before this feature
                    if !warned {
                        tracing::warn!(error = %e, "data-change shipper: cannot read al_data_change");
                        warned = true;
                    }
                }
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    });
}
