//! Localization Commands Interface (Trait Contracts for Routes).
//!
//! Provides the `LocalizationCommands` trait interface for HTTP routes to execute i18n operations.

pub mod get_i18n;
pub mod get_i18n_by_code;
pub mod i18n_executor;

pub use get_i18n::{GetI18nCommand, GetI18nCommandHandler};
pub use get_i18n_by_code::{GetI18nByCodeCommand, GetI18nByCodeCommandHandler};
pub use i18n_executor::{I18nCommandExecutor, LocalizationCommandService};

use crate::features::localization::errors::LocalizationError;
use crate::features::localization::models::I18nDictionary;
use async_trait::async_trait;
use uuid::Uuid;

/// Trait interface for Localization commands (called by API routes).
#[async_trait]
pub trait LocalizationCommands: Send + Sync {
    /// Retrieve i18n UI translation dictionary for the given language ID (`db_language.id` / lid).
    async fn get_i18n(&self, language_id: Uuid) -> Result<I18nDictionary, LocalizationError>;

    /// Retrieve the dictionary by language code (`th`, `en`; case-insensitive).
    /// `LanguageNotFound` when the code is unknown.
    async fn get_i18n_by_code(&self, code: &str) -> Result<I18nDictionary, LocalizationError>;
}

/// Generic Command Handler trait interface.
#[async_trait]
pub trait CommandHandler<C, R>: Send + Sync {
    async fn handle(&self, command: C) -> Result<R, LocalizationError>;
}
