//! RPPG000 — My reports (รายงานของฉัน): report requests the worker builds in the background.
//!
//! Flow: the API publishes a [`ReportJobMessage`](application_core::traits::ReportJobMessage) to
//! Redpanda; the worker (`api-report`) feeds it to [`Rppg000Commands::run_job`], which streams
//! the CSV from the audit store into object storage and then notifies the requester. The same
//! feature serves the requester's list, a preview of the first rows and the download link.

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::{ReportJobHandler, Rppg000CommandExecutor, Rppg000Commands, preview_rows};
pub use errors::Rppg000Error;
pub use models::*;
pub use queries::{DbRppg000Queries, Rppg000Queries};
