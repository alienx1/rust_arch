//! DBPG005 — Geography (จังหวัด / อำเภอ / ตำบล / รหัสไปรษณีย์).
//!
//! Program code `DBPG005`, API `/api/v1/db/dbpg005`. Paged, filterable lists per level (province,
//! district, sub-district, postal code) plus create/update.

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::{Dbpg005CommandExecutor, Dbpg005Commands, normalize_postal_code};
pub use errors::Dbpg005Error;
pub use models::*;
pub use queries::{DbDbpg005Queries, Dbpg005Queries};
