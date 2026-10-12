//! DBPG003 — Employees (ทะเบียนพนักงาน): paged list + create/update.
//!
//! Program code `DBPG003`, API `/api/v1/db/dbpg003`.

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::{Dbpg003CommandExecutor, Dbpg003Commands};
pub use errors::Dbpg003Error;
pub use models::*;
pub use queries::{DbDbpg003Queries, Dbpg003Queries};
