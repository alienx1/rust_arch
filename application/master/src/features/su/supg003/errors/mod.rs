//! SUPG003 feature errors

use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Supg003Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Duplicate code error: {0}")]
    DuplicateCode(String),

    #[error("Optimistic lock conflict: row version mismatch")]
    VersionConflict,

    #[error("Validation error: {0}")]
    Validation(String),

    /// Something still references the row (users in the group, foreign keys)
    #[error("In use: {0}")]
    InUse(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<DbErr> for Supg003Error {
    fn from(err: DbErr) -> Self {
        Supg003Error::Database(err.to_string())
    }
}
