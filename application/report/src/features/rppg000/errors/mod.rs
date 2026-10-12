//! RPPG000 feature errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Rppg000Error {
    /// Bad request (maps to 400)
    #[error("Validation error: {0}")]
    Validation(String),

    /// No such job, or it belongs to somebody else (maps to 404)
    #[error("Not found: {0}")]
    NotFound(String),

    /// The job exists but has no file (yet): still running, or failed (maps to 409)
    #[error("Not ready: {0}")]
    NotReady(String),

    /// The job succeeded but its file is gone from storage (deleted, or the storage was wiped):
    /// maps to 410
    #[error("File gone: {0}")]
    FileGone(String),

    /// Object storage cannot be reached (maps to 503)
    #[error("Storage unavailable: {0}")]
    Storage(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
