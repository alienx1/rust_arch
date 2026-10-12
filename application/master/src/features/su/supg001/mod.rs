//! SUPG001 — Systems & Modules (จัดการระบบ / โมดูล): `db_system` / `db_module` + labels.
//!
//! Program code `SUPG001`, API `/api/v1/su/supg001`. Switching a system or module off hides every
//! program under it from the menu (the menu tree only includes programs of active systems and
//! modules).

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::{Supg001CommandExecutor, Supg001Commands, normalize_unit_code};
pub use errors::Supg001Error;
pub use models::*;
pub use queries::{DbSupg001Queries, Supg001Queries};
