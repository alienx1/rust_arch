//! Tests for the cached i18n dictionaries and their refill after a label write.
//! Located at `application/master/tests/cached_localization_test.rs`.

use application_core::traits::{CacheRefresher, CacheService, InMemoryCache, ReadCaches, cache_ns};
use application_master::features::localization::{
    CachedLocalizationQueries, I18nCacheRefresher, I18nDictionary, LocalizationError, LocalizationQueries,
};
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use uuid::Uuid;

/// The "database": counts reads, and its dictionary can be edited
struct Labels {
    reads: AtomicUsize,
    value: std::sync::Mutex<String>,
    thai: Uuid,
    english: Uuid,
}

impl Labels {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            reads: AtomicUsize::new(0),
            value: std::sync::Mutex::new("สวัสดี".into()),
            thai: Uuid::now_v7(),
            english: Uuid::now_v7(),
        })
    }
}

#[async_trait]
impl LocalizationQueries for Labels {
    async fn get_i18n_dictionary(&self, language_id: Uuid) -> Result<I18nDictionary, LocalizationError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        let mut dict = I18nDictionary::new(language_id);
        dict.insert("common.hello", self.value.lock().unwrap().clone());
        Ok(dict)
    }
    async fn get_language_id_by_code(&self, code: &str) -> Result<Option<Uuid>, LocalizationError> {
        Ok(match code {
            "th" => Some(self.thai),
            "en" => Some(self.english),
            _ => None,
        })
    }
}

fn cached(labels: Arc<Labels>) -> (Arc<InMemoryCache>, Arc<CachedLocalizationQueries>) {
    let memory = Arc::new(InMemoryCache::new());
    let caches = ReadCaches::new(Some(memory.clone() as Arc<dyn CacheService>));
    (memory, Arc::new(CachedLocalizationQueries::new(labels, caches.ns(cache_ns::I18N))))
}

#[tokio::test]
async fn test_a_dictionary_is_read_from_the_database_once() {
    let labels = Labels::new();
    let (_, queries) = cached(labels.clone());
    for _ in 0..5 {
        assert_eq!(queries.get_i18n_dictionary(labels.thai).await.unwrap().labels["common.hello"], "สวัสดี");
    }
    assert_eq!(labels.reads.load(Ordering::SeqCst), 1);
    queries.get_i18n_dictionary(labels.english).await.unwrap();
    assert_eq!(labels.reads.load(Ordering::SeqCst), 2, "one entry per language");
}

#[tokio::test]
async fn test_language_codes_resolve_case_insensitively_through_one_entry() {
    let labels = Labels::new();
    let (memory, queries) = cached(labels.clone());
    assert_eq!(queries.get_language_id_by_code("TH").await.unwrap(), Some(labels.thai));
    assert_eq!(queries.get_language_id_by_code(" th ").await.unwrap(), Some(labels.thai));
    assert_eq!(memory.keys("arch:i18n:").len(), 1);
    assert_eq!(queries.get_language_id_by_code("xx").await.unwrap(), None);
}

#[tokio::test]
async fn test_after_a_label_write_the_refill_puts_the_new_text_in_valkey() {
    let labels = Labels::new();
    let (memory, queries) = cached(labels.clone());
    queries.get_i18n_dictionary(labels.thai).await.unwrap();

    // DBPG006 saves a label: the API clears the namespace, then refills it
    *labels.value.lock().unwrap() = "ดีจ้า".into();
    assert!(application_core::traits::ReadCache::invalidate_namespace(memory.as_ref(), cache_ns::I18N).await > 0);
    assert!(memory.keys("arch:i18n:").is_empty());
    I18nCacheRefresher::new(queries.clone()).refresh(cache_ns::I18N).await;
    assert!(!memory.keys("arch:i18n:").is_empty(), "refilled without waiting for a reader");

    let reads_after_refill = labels.reads.load(Ordering::SeqCst);
    assert_eq!(queries.get_i18n_dictionary(labels.thai).await.unwrap().labels["common.hello"], "ดีจ้า");
    assert_eq!(labels.reads.load(Ordering::SeqCst), reads_after_refill, "served from the refilled entry");
}

#[tokio::test]
async fn test_the_refresher_ignores_other_namespaces() {
    let labels = Labels::new();
    let (memory, queries) = cached(labels.clone());
    I18nCacheRefresher::new(queries).refresh(cache_ns::REF).await;
    assert!(memory.is_empty());
    assert_eq!(labels.reads.load(Ordering::SeqCst), 0);
}
