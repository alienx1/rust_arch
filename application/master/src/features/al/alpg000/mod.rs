//! ALPG000 — Audit Log (บันทึกการตรวจสอบ): who opened which page, pressed which button, ran which
//! process, and the system log of every API call (request / response, status, headers).
//!
//! Program code `ALPG000`, API `/api/v1/al/alpg000`. Events live in Quickwit behind
//! `AuditLogPort`; this feature validates the search filter, shapes the rows for the page and
//! accepts the page / click / action events reported by the web app (`POST /api/v1/audit/events`).

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::{
    Alpg000CommandExecutor, Alpg000Commands, normalize_client_events, normalize_search,
};
pub use errors::Alpg000Error;
pub use queries::{Alpg000Queries, DbAlpg000Queries};
pub use models::*;
