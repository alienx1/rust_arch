//! Access audit tracking (page views, clicks, actions) reported to `POST /api/v1/audit/events`.
//!
//! Fire-and-forget: events are queued and flushed in batches; failures are never shown.

pub mod audit_click_listener;
pub mod audit_event;
pub mod audit_tracker;
pub mod audit_tracking;

pub use audit_tracker::track_action;
pub use audit_tracking::use_audit_tracking;
