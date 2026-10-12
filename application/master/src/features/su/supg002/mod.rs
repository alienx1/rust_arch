//! SUPG002 — Menus & Programs: the sidebar menu tree from `db_program`, filtered by permission.
//!
//! Program code `SUPG002`, API `/api/v1/su/supg002`. Read-only for now (menu management screens
//! come later); a menu entry is switched on/off with `db_program.active` (or by revoking READ).

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::{
    Supg002CommandExecutor, Supg002Commands, normalize_program_code, normalize_program_path,
};
pub use errors::Supg002Error;
pub use models::*;
pub use queries::{DbSupg002Queries, Supg002Queries, build_menu_tree};
