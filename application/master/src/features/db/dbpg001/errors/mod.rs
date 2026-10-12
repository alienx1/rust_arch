//! DBPG001 feature errors

use sea_orm::DbErr;
use thiserror::Error;

use crate::features::db::dbpg000::errors::Dbpg000Error;

#[derive(Debug, Error)]
pub enum Dbpg001Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Organization not found: {0}")]
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

impl From<DbErr> for Dbpg001Error {
    fn from(err: DbErr) -> Self {
        Dbpg001Error::Database(err.to_string())
    }
}

impl From<Dbpg000Error> for Dbpg001Error {
    fn from(err: Dbpg000Error) -> Self {
        match err {
            Dbpg000Error::Database(m) => Self::Database(m),
            Dbpg000Error::NotFound(m) => Self::NotFound(m),
            Dbpg000Error::DuplicateCode(m) => Self::DuplicateCode(m),
            Dbpg000Error::VersionConflict => Self::VersionConflict,
            Dbpg000Error::Validation(m) => Self::Validation(m),
            Dbpg000Error::Internal(m) => Self::Internal(m),
        }
    }
}
