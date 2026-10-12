//! Executor implementing `LocalizationCommands` for route handlers.

use crate::features::localization::commands::LocalizationCommands;
use crate::features::localization::errors::LocalizationError;
use crate::features::localization::models::I18nDictionary;
use crate::features::localization::queries::LocalizationQueries;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

/// Executes i18n commands by coordinating with the query engine.
#[derive(Clone)]
pub struct I18nCommandExecutor {
    queries: Arc<dyn LocalizationQueries>,
}

impl I18nCommandExecutor {
    pub fn new(queries: Arc<dyn LocalizationQueries>) -> Self {
        Self { queries }
    }
}

// Backward-compatible alias
pub type LocalizationCommandService = I18nCommandExecutor;

#[async_trait]
impl LocalizationCommands for I18nCommandExecutor {
    async fn get_i18n(&self, language_id: Uuid) -> Result<I18nDictionary, LocalizationError> {
        self.queries.get_i18n_dictionary(language_id).await
    }

    async fn get_i18n_by_code(&self, code: &str) -> Result<I18nDictionary, LocalizationError> {
        let language_id = self
            .queries
            .get_language_id_by_code(code)
            .await?
            .ok_or_else(|| LocalizationError::LanguageNotFound(code.to_string()))?;
        self.queries.get_i18n_dictionary(language_id).await
    }
}
