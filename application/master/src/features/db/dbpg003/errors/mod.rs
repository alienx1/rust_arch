//! DBPG003 feature errors

use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Dbpg003Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Duplicate code error: {0}")]
    DuplicateCode(String),

    #[error("Optimistic lock conflict: row version mismatch")]
    VersionConflict,

    #[error("In use: {0}")]
    InUse(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<DbErr> for Dbpg003Error {
    fn from(err: DbErr) -> Self {
        Dbpg003Error::Database(err.to_string())
    }
}
