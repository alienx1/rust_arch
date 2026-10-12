//! ALPG000 feature errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Alpg000Error {
    /// Bad filter or request (maps to 400)
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    /// The audit store (Quickwit) cannot be reached (maps to 503)
    #[error("Audit store unavailable: {0}")]
    Unavailable(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
