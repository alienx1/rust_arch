//! ALPG001 feature errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Alpg001Error {
    /// Bad filter or request (maps to 400)
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    /// The process cannot be reprocessed right now, e.g. it is running (maps to 409)
    #[error("Conflict: {0}")]
    Conflict(String),

    /// The broker cannot be reached (maps to 503)
    #[error("Process queue unavailable: {0}")]
    Unavailable(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
