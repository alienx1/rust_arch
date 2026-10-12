//! Quickwit adapter for the audit log (`AuditLogPort`).
//!
//! - `quickwit_config`: `QUICKWIT_URL` / `AUDIT_LOG_ENABLED`
//! - `index_bootstrap`: creates the `audit-logs-v1` index when it is missing (self-healing)
//! - `audit_query`: pure query / document builders (unit-tested)
//! - `event_stream_query`: the same for the Redpanda event indexes (Kafka source)
//! - `audit_flusher`: bounded queue + the single background flusher (NDJSON ingest)
//! - `quickwit_audit_log`: the `AuditLogPort` implementation (record / search / get / health)

pub mod audit_flusher;
pub mod data_change_event;
pub mod data_change_shipper;
pub mod audit_query;
pub mod event_stream_query;
pub mod index_bootstrap;
pub mod quickwit_client;
pub mod quickwit_audit_log;
pub mod quickwit_config;

pub use audit_query::{
    AuditTimeRange, audit_time_range, build_audit_query, doc_to_event, event_to_doc, export_search_body, page_window,
};
pub use event_stream_query::{build_event_query, event_doc_to_audit, event_index_for, is_event_kind};
pub use data_change_event::{DataChangeRow, changes_to_event, group_changes};
pub use data_change_shipper::{ship_once, spawn_data_change_shipper};
pub use quickwit_audit_log::QuickwitAuditLog;
pub use quickwit_config::QuickwitConfig;
