//! Source of API error message templates (`db_label` rows with category `error`).

use crate::features::localization::errors::LocalizationError;
use crate::features::localization::queries::LocalizationQueries;
use async_trait::async_trait;
pub use application_core::traits::{ErrorTemplateError, ErrorTemplateSource};
use std::collections::HashMap;
use std::sync::Arc;

/// Label category holding the error templates; the label key is the API error code.
const ERROR_CATEGORY_PREFIX: &str = "error.";

/// `ErrorTemplateSource` backed by the localization queries (`db_label` / `db_language`).
#[derive(Clone)]
pub struct DbErrorTemplateSource {
    queries: Arc<dyn LocalizationQueries>,
}

impl DbErrorTemplateSource {
    pub fn new(queries: Arc<dyn LocalizationQueries>) -> Self {
        Self { queries }
    }
}

#[async_trait]
impl ErrorTemplateSource for DbErrorTemplateSource {
    async fn error_templates(
        &self,
        language_code: &str,
    ) -> Result<HashMap<String, String>, ErrorTemplateError> {
        let failed = |e: LocalizationError| ErrorTemplateError(e.to_string());
        let language_id = self
            .queries
            .get_language_id_by_code(language_code)
            .await
            .map_err(failed)?
            .ok_or_else(|| {
                ErrorTemplateError(LocalizationError::LanguageNotFound(language_code.to_string()).to_string())
            })?;
        let dict = self.queries.get_i18n_dictionary(language_id).await.map_err(failed)?;
        Ok(dict
            .labels
            .into_iter()
            .filter_map(|(key, value)| {
                key.strip_prefix(ERROR_CATEGORY_PREFIX)
                    .map(|code| (code.to_string(), value))
            })
            .collect())
    }
}
