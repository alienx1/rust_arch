//! Localization read query error definitions.

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LocalizationError {
    #[error("Language not found: {0}")]
    LanguageNotFound(String),

    #[error("Label not found: {0}")]
    LabelNotFound(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(String),
}
