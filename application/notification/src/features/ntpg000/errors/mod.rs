//! NTPG000 feature errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Ntpg000Error {
    /// No such notification (or somebody else's): maps to 404
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
