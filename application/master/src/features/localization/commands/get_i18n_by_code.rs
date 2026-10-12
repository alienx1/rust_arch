//! Get i18n Dictionary By Language Code Command & Handler.

use crate::features::localization::commands::{CommandHandler, LocalizationCommands};
use crate::features::localization::errors::LocalizationError;
use crate::features::localization::models::I18nDictionary;
use async_trait::async_trait;
use std::sync::Arc;

/// Command invoked by API routes to fetch the i18n dictionary for a language code (`th`, `en`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GetI18nByCodeCommand {
    /// Language code (`db_language.code`), case-insensitive
    pub code: String,
}

impl GetI18nByCodeCommand {
    pub fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }
}

/// Command Handler executing `GetI18nByCodeCommand`.
pub struct GetI18nByCodeCommandHandler {
    commands: Arc<dyn LocalizationCommands>,
}

impl GetI18nByCodeCommandHandler {
    pub fn new(commands: Arc<dyn LocalizationCommands>) -> Self {
        Self { commands }
    }
}

#[async_trait]
impl CommandHandler<GetI18nByCodeCommand, I18nDictionary> for GetI18nByCodeCommandHandler {
    async fn handle(&self, cmd: GetI18nByCodeCommand) -> Result<I18nDictionary, LocalizationError> {
        self.commands.get_i18n_by_code(&cmd.code).await
    }
}
