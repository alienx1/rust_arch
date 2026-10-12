//! DBPG006 — i18n Labels (ข้อความแปลภาษา): the translation dictionary in `db_label`.
//!
//! Program code `DBPG006`, API `/api/v1/db/dbpg006`. One entry is a `(category, key)` pair with one
//! row per language; the list pages over entries, create/update write all languages of an entry
//! in one transaction. The read-only dictionary endpoint (`features::localization`) reads the
//! table (through the Valkey `i18n` cache, which every write here clears and refills), so edits here show up there at once.

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::{
    Dbpg006CommandExecutor, Dbpg006Commands, normalize_label_category, normalize_label_key,
};
pub use errors::Dbpg006Error;
pub use models::*;
pub use queries::{DbDbpg006Queries, Dbpg006Queries};
