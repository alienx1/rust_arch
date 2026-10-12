//! DBPG001 — Organizations & Companies (detail forms): write-side feature.
//!
//! Program code `DBPG001`, API `/api/v1/db/dbpg001`. List/summary reads live in `dbpg000`.

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::{Dbpg001CommandExecutor, Dbpg001Commands};
pub use errors::Dbpg001Error;
pub use models::*;
pub use queries::{DbDbpg001Queries, Dbpg001Queries};
