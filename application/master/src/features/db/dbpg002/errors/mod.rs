//! DBPG002 feature errors

use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Dbpg002Error {
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

    /// The caller may not do this part of the request (e.g. hand out a permission group)
    #[error("Not allowed: {0}")]
    NotAllowed(String),

    #[error("In use: {0}")]
    InUse(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<DbErr> for Dbpg002Error {
    fn from(err: DbErr) -> Self {
        match err {
            // `update_versioned` found the row at another version (or gone): somebody saved first
            DbErr::RecordNotUpdated => Dbpg002Error::VersionConflict,
            other => Dbpg002Error::Database(other.to_string()),
        }
    }
}
