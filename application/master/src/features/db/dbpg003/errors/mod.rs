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
        match err {
            // `update_versioned` found the row at another version (or gone): somebody saved first
            DbErr::RecordNotUpdated => Dbpg003Error::VersionConflict,
            other => Dbpg003Error::Database(other.to_string()),
        }
    }
}
