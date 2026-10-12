//! Localization Queries Interface (Reads).
//!
//! Exclusively scoped for i18n frontend translation lookups.
//! Provides the dictionary query taking `language_id` (lid of `db_language`) and a language-code lookup.

pub mod db_label_queries;
pub mod error_template_source;
pub mod get_i18n_dictionary;

pub use db_label_queries::{DbLabelQueries, LocalizationQueryService};
mod cached_label_queries;
pub use cached_label_queries::{CachedLocalizationQueries, I18nCacheRefresher};
pub use error_template_source::{DbErrorTemplateSource, ErrorTemplateSource};
pub use get_i18n_dictionary::{GetI18nDictionaryHandler, GetI18nDictionaryQuery};

use crate::features::localization::errors::LocalizationError;
use crate::features::localization::models::I18nDictionary;
use async_trait::async_trait;
use uuid::Uuid;

/// Trait interface for Localization i18n queries.
/// Exclusively provides the single query needed to load the translation dictionary.
#[async_trait]
pub trait LocalizationQueries: Send + Sync {
    /// Retrieve the complete UI translation dictionary for the given language ID (`db_language.id` / lid).
    async fn get_i18n_dictionary(
        &self,
        language_id: Uuid,
    ) -> Result<I18nDictionary, LocalizationError>;

    /// Resolve a language code (`db_language.code`, matched case-insensitively) to its ID.
    /// `None` when no such language exists.
    async fn get_language_id_by_code(&self, code: &str) -> Result<Option<Uuid>, LocalizationError>;
}

/// Generic Query Handler trait interface.
#[async_trait]
pub trait QueryHandler<Q, R>: Send + Sync {
    async fn handle(&self, query: Q) -> Result<R, LocalizationError>;
}
