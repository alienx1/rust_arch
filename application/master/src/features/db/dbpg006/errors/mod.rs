//! DBPG006 feature errors

use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Dbpg006Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Duplicate label: {0}")]
    Duplicate(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<DbErr> for Dbpg006Error {
    fn from(err: DbErr) -> Self {
        Dbpg006Error::Database(err.to_string())
    }
}
