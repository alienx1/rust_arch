//! Get i18n Dictionary Query & Handler.

use crate::features::localization::errors::LocalizationError;
use crate::features::localization::models::I18nDictionary;
use crate::features::localization::queries::{LocalizationQueries, QueryHandler};
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

/// Query to fetch the complete i18n UI translation dictionary for a given `language_id` (lid of `db_language`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GetI18nDictionaryQuery {
    /// Language ID (`db_language.id` / lid)
    pub language_id: Uuid,
}

impl GetI18nDictionaryQuery {
    pub fn new(language_id: Uuid) -> Self {
        Self { language_id }
    }
}

pub struct GetI18nDictionaryHandler {
    queries: Arc<dyn LocalizationQueries>,
}

impl GetI18nDictionaryHandler {
    pub fn new(queries: Arc<dyn LocalizationQueries>) -> Self {
        Self { queries }
    }
}

#[async_trait]
impl QueryHandler<GetI18nDictionaryQuery, I18nDictionary> for GetI18nDictionaryHandler {
    async fn handle(
        &self,
        query: GetI18nDictionaryQuery,
    ) -> Result<I18nDictionary, LocalizationError> {
        self.queries.get_i18n_dictionary(query.language_id).await
    }
}
