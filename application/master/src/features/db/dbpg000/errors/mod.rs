//! DBPG000 feature errors

use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Dbpg000Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Organization not found: {0}")]
    NotFound(String),

    #[error("Duplicate code error: {0}")]
    DuplicateCode(String),

    #[error("Optimistic lock conflict: row version mismatch")]
    VersionConflict,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<DbErr> for Dbpg000Error {
    fn from(err: DbErr) -> Self {
        Dbpg000Error::Database(err.to_string())
    }
}
