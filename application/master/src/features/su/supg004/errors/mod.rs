//! SUPG004 feature errors

use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Supg004Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Duplicate error: {0}")]
    Duplicate(String),

    #[error("Optimistic lock conflict: row version mismatch")]
    VersionConflict,

    #[error("Validation error: {0}")]
    Validation(String),

    /// The change is not allowed for the acting user (self-lockout, granting ADMIN)
    #[error("Not allowed: {0}")]
    NotAllowed(String),

    /// The identity provider (Ory Kratos) refused or could not be reached
    #[error("Identity provider error: {0}")]
    IdentityProvider(String),

    /// Something still references the row (foreign keys)
    #[error("In use: {0}")]
    InUse(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<DbErr> for Supg004Error {
    fn from(err: DbErr) -> Self {
        match err {
            // `update_versioned` found the row at another version (or gone): somebody saved first
            DbErr::RecordNotUpdated => Supg004Error::VersionConflict,
            other => Supg004Error::Database(other.to_string()),
        }
    }
}
