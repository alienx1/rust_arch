//! Account security errors

use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccountSecurityError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    /// The identity provider failed or has no such account
    #[error("Identity provider error: {0}")]
    Identity(String),

    #[error("Database error: {0}")]
    Database(String),
}

impl From<DbErr> for AccountSecurityError {
    fn from(err: DbErr) -> Self {
        AccountSecurityError::Database(err.to_string())
    }
}
