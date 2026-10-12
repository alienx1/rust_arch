//! Referential-integrity helper shared by the delete actions.
//!
//! A delete removes the row's owned data (labels, permission rows, ...) and then the row itself in
//! one transaction. When something else still references the row, PostgreSQL refuses with a
//! foreign-key violation; the transaction rolls back and the action reports "in use" instead of
//! a database error.

use sea_orm::{DbErr, SqlErr};

/// `true` when the database refused a write because another row still references the target
pub fn is_foreign_key_violation(err: &DbErr) -> bool {
    matches!(err.sql_err(), Some(SqlErr::ForeignKeyConstraintViolation(_)))
}
