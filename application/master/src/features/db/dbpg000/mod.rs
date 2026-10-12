//! DBPG000 — Organizations & Companies (main list screen): read-side feature.
//!
//! Program code `DBPG000`, API `/api/v1/db/dbpg000`. Create/update lives in `dbpg001`.

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::{Dbpg000CommandExecutor, Dbpg000Commands};
pub use errors::Dbpg000Error;
pub use models::*;
pub use queries::{DbDbpg000Queries, Dbpg000Queries};
