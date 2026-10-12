//! DBPG004 — Reference Data (ข้อมูลอ้างอิงระบบ): `db_master_lookup` + `db_master_lookup_label`.
//!
//! Program code `DBPG004`, API `/api/v1/db/dbpg004`. Paged list per category plus create/update.

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::{Dbpg004CommandExecutor, Dbpg004Commands, normalize_category, normalize_code};
pub use errors::Dbpg004Error;
pub use models::*;
pub use queries::{DbDbpg004Queries, Dbpg004Queries};
