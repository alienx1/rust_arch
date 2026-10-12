//! `LocalizationQueries` with a Valkey read-through in front of `db_label` / `db_language`.
//!
//! One entry per language holds the whole dictionary (~1,200 labels); the web app fetches it on
//! every page load and the API error middleware builds its templates from it, so both read the
//! same cached copy. A write to the labels (DBPG006) clears the namespace and refills it at once
//! ([`I18nCacheRefresher`]).

use application_core::traits::{CacheRefresher, ReadCache, cache_ns, cache_ttl};
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use super::LocalizationQueries;
use crate::features::localization::errors::LocalizationError;
use crate::features::localization::models::I18nDictionary;

/// Languages the dictionary is refilled for after a label write
const REFILLED_LANGUAGES: [&str; 2] = ["th", "en"];

pub struct CachedLocalizationQueries {
    inner: Arc<dyn LocalizationQueries>,
    cache: ReadCache,
}

impl CachedLocalizationQueries {
    /// Dictionaries do not depend on the request language, so the cache is language-free
    pub fn new(inner: Arc<dyn LocalizationQueries>, cache: ReadCache) -> Self {
        Self { inner, cache: cache.language_free() }
    }
}

#[async_trait]
impl LocalizationQueries for CachedLocalizationQueries {
    async fn get_i18n_dictionary(&self, language_id: Uuid) -> Result<I18nDictionary, LocalizationError> {
        self.cache
            .through("dictionary", &language_id, cache_ttl::I18N, || self.inner.get_i18n_dictionary(language_id))
            .await
    }

    async fn get_language_id_by_code(&self, code: &str) -> Result<Option<Uuid>, LocalizationError> {
        let code = code.trim().to_lowercase();
        self.cache
            .through("language_id", &code, cache_ttl::I18N, || self.inner.get_language_id_by_code(&code))
            .await
    }
}

/// Refills the dictionaries after a label write so the next page load is a cache hit
pub struct I18nCacheRefresher {
    queries: Arc<dyn LocalizationQueries>,
}

impl I18nCacheRefresher {
    /// `queries` must be the cached ones: reading through them is what refills the cache
    pub fn new(queries: Arc<dyn LocalizationQueries>) -> Self {
        Self { queries }
    }
}

#[async_trait]
impl CacheRefresher for I18nCacheRefresher {
    async fn refresh(&self, namespace: &str) {
        if namespace != cache_ns::I18N {
            return;
        }
        let mut refilled = Vec::new();
        for code in REFILLED_LANGUAGES {
            match self.queries.get_language_id_by_code(code).await {
                Ok(Some(id)) => match self.queries.get_i18n_dictionary(id).await {
                    Ok(dictionary) => refilled.push(format!("{code} ({} labels)", dictionary.labels.len())),
                    Err(e) => tracing::warn!(target: "arch::cache", code, error = %e, "i18n refill failed"),
                },
                Ok(None) => {}
                Err(e) => tracing::warn!(target: "arch::cache", code, error = %e, "i18n refill failed"),
            }
        }
        tracing::info!(target: "arch::cache", "REFILL i18n: {}", refilled.join(", "));
    }
}
