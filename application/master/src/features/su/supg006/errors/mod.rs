//! SUPG006 feature errors

use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Supg006Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Optimistic lock conflict: row version mismatch")]
    VersionConflict,

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<DbErr> for Supg006Error {
    fn from(err: DbErr) -> Self {
        match err {
            // `update_versioned` found the row at another version (or gone): somebody saved first
            DbErr::RecordNotUpdated => Supg006Error::VersionConflict,
            other => Supg006Error::Database(other.to_string()),
        }
    }
}
