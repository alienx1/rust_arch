//! SUPG001 feature errors

use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Supg001Error {
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

    #[error("In use: {0}")]
    InUse(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<DbErr> for Supg001Error {
    fn from(err: DbErr) -> Self {
        match err {
            // `update_versioned` found the row at another version (or gone): somebody saved first
            DbErr::RecordNotUpdated => Supg001Error::VersionConflict,
            other => Supg001Error::Database(other.to_string()),
        }
    }
}
