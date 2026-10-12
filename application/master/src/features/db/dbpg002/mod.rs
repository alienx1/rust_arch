//! DBPG002 — Branches & Departments (list + create/update).
//!
//! Program code `DBPG002`, API `/api/v1/db/dbpg002`.

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::{Dbpg002CommandExecutor, Dbpg002Commands};
pub use errors::Dbpg002Error;
pub use models::*;
pub use queries::{DbDbpg002Queries, Dbpg002Queries};
