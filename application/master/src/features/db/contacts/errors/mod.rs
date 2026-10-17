//! Contact feature errors

use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContactError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    /// The sign-in e-mail is already used by another account
    #[error("Conflict: {0}")]
    Conflict(String),

    /// The identity provider (Kratos) could not apply the change
    #[error("Identity provider error: {0}")]
    Identity(String),

    #[error("Database error: {0}")]
    Database(String),
}

impl From<DbErr> for ContactError {
    fn from(err: DbErr) -> Self {
        ContactError::Database(err.to_string())
    }
}
