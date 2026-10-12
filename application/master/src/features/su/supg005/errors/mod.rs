//! IAM Feature Error definitions

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Supg005Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Identity store (Ory Kratos) error: {0}")]
    Kratos(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Internal IAM service error: {0}")]
    Internal(String),
}

impl From<sea_orm::DbErr> for Supg005Error {
    fn from(err: sea_orm::DbErr) -> Self {
        Self::Database(err.to_string())
    }
}
