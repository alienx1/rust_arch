//! Get i18n Dictionary Command & Handler.

use crate::features::localization::commands::{CommandHandler, LocalizationCommands};
use crate::features::localization::errors::LocalizationError;
use crate::features::localization::models::I18nDictionary;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

/// Command invoked by API routes to fetch the i18n dictionary for a given `language_id` (lid).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GetI18nCommand {
    /// Language ID (`db_language.id` / lid)
    pub language_id: Uuid,
}

impl GetI18nCommand {
    pub fn new(language_id: Uuid) -> Self {
        Self { language_id }
    }
}

/// Command Handler executing `GetI18nCommand`.
pub struct GetI18nCommandHandler {
    commands: Arc<dyn LocalizationCommands>,
}

impl GetI18nCommandHandler {
    pub fn new(commands: Arc<dyn LocalizationCommands>) -> Self {
        Self { commands }
    }
}

#[async_trait]
impl CommandHandler<GetI18nCommand, I18nDictionary> for GetI18nCommandHandler {
    async fn handle(&self, cmd: GetI18nCommand) -> Result<I18nDictionary, LocalizationError> {
        self.commands.get_i18n(cmd.language_id).await
    }
}
