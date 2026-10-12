//! Tests for the Valkey read-through cache (`ReadCache`) over the in-memory cache service.
//! Located at `application/core/tests/read_cache_test.rs`.

use application_core::errors::CacheError;
use application_core::request_language::{self, Language};
use application_core::traits::{CacheService, InMemoryCache, ReadCaches, cache_ns};
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn caches() -> (Arc<InMemoryCache>, ReadCaches) {
    let memory = Arc::new(InMemoryCache::new());
    (memory.clone(), ReadCaches::new(Some(memory as Arc<dyn CacheService>)))
}

/// Counts how often the "database" is asked
#[derive(Default, Clone)]
struct Loads(Arc<AtomicUsize>);

impl Loads {
    async fn answer(&self, value: u32) -> Result<u32, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(value)
    }
    fn count(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }
}

#[tokio::test]
async fn test_second_read_is_a_hit_and_never_reaches_the_database() {
    let (_, caches) = caches();
    let cache = caches.ns(cache_ns::REF);
    let loads = Loads::default();
    for _ in 0..3 {
        let v = cache.through("provinces", &"th", 60, || loads.answer(7)).await.unwrap();
        assert_eq!(v, 7);
    }
    assert_eq!(loads.count(), 1);
}

#[tokio::test]
async fn test_different_arguments_are_different_entries() {
    let (memory, caches) = caches();
    let cache = caches.ns(cache_ns::REF);
    let loads = Loads::default();
    assert_eq!(cache.through("districts", &"a", 60, || loads.answer(1)).await.unwrap(), 1);
    assert_eq!(cache.through("districts", &"b", 60, || loads.answer(2)).await.unwrap(), 2);
    assert_eq!(cache.through("districts", &"a", 60, || loads.answer(99)).await.unwrap(), 1);
    assert_eq!(loads.count(), 2);
    assert_eq!(memory.keys("arch:ref:").len(), 2);
}

#[tokio::test]
async fn test_errors_are_returned_and_never_cached() {
    let (memory, caches) = caches();
    let cache = caches.ns(cache_ns::ORG);
    let failed: Result<u32, String> = cache.through("x", &(), 60, || async { Err("db down".to_string()) }).await;
    assert_eq!(failed, Err("db down".to_string()));
    assert!(memory.is_empty());
    let loads = Loads::default();
    assert_eq!(cache.through("x", &(), 60, || loads.answer(5)).await.unwrap(), 5, "the next read loads again");
}

#[tokio::test]
async fn test_answers_follow_the_request_language_unless_the_namespace_is_language_free() {
    let (_, caches) = caches();
    let cache = caches.ns(cache_ns::PROGRAMS);
    let thai = request_language::scope(Language::Thai, async { cache.key("options", &()) }).await;
    let english = request_language::scope(Language::English, async { cache.key("options", &()) }).await;
    assert_ne!(thai, english);
    assert!(thai.starts_with("arch:programs:v1:th:"));

    let free = caches.ns(cache_ns::I18N).language_free();
    let a = request_language::scope(Language::Thai, async { free.key("dictionary", &1) }).await;
    let b = request_language::scope(Language::English, async { free.key("dictionary", &1) }).await;
    assert_eq!(a, b);
}

#[tokio::test]
async fn test_keys_are_stable_across_instances() {
    let (_, one) = caches();
    let (_, two) = caches();
    let args = serde_json::json!({ "page": 2, "q": "ก" });
    assert_eq!(one.ns(cache_ns::REF).key("list", &args), two.ns(cache_ns::REF).key("list", &args));
}

#[tokio::test]
async fn test_invalidate_drops_only_its_namespace() {
    let (memory, caches) = caches();
    let loads = Loads::default();
    caches.ns(cache_ns::REF).through("a", &(), 60, || loads.answer(1)).await.unwrap();
    caches.ns(cache_ns::ORG).through("a", &(), 60, || loads.answer(1)).await.unwrap();
    assert_eq!(caches.ns(cache_ns::REF).invalidate().await, 1);
    assert!(memory.keys("arch:ref:").is_empty());
    assert_eq!(memory.keys("arch:org:").len(), 1);
    // After the clear the next read loads again and so refreshes the entry
    caches.ns(cache_ns::REF).through("a", &(), 60, || loads.answer(2)).await.unwrap();
    assert_eq!(loads.count(), 3);
}

#[tokio::test]
async fn test_put_replaces_what_readers_see() {
    let (_, caches) = caches();
    let cache = caches.ns(cache_ns::REF);
    let loads = Loads::default();
    assert_eq!(cache.through("n", &(), 60, || loads.answer(1)).await.unwrap(), 1);
    cache.put("n", &(), 60, &42u32).await;
    assert_eq!(cache.through("n", &(), 60, || loads.answer(1)).await.unwrap(), 42);
    assert_eq!(loads.count(), 1);
}

#[tokio::test]
async fn test_a_cached_value_of_the_wrong_shape_is_reloaded() {
    let (memory, caches) = caches();
    let cache = caches.ns(cache_ns::REF);
    let key = cache.key("shape", &());
    memory.set(&key, "{\"not\":\"a number\"}", Some(60)).await.unwrap();
    let loads = Loads::default();
    assert_eq!(cache.through("shape", &(), 60, || loads.answer(3)).await.unwrap(), 3);
    assert_eq!(memory.get(&key).await.unwrap().as_deref(), Some("3"), "the bad entry was replaced");
}

/// A Valkey that is down: every call fails
struct BrokenCache;

#[async_trait]
impl CacheService for BrokenCache {
    async fn get(&self, _: &str) -> Result<Option<String>, CacheError> {
        Err(CacheError::OperationFailed("down".into()))
    }
    async fn set(&self, _: &str, _: &str, _: Option<u64>) -> Result<(), CacheError> {
        Err(CacheError::OperationFailed("down".into()))
    }
    async fn del(&self, _: &str) -> Result<(), CacheError> {
        Err(CacheError::OperationFailed("down".into()))
    }
    async fn del_prefix(&self, _: &str) -> Result<u64, CacheError> {
        Err(CacheError::OperationFailed("down".into()))
    }
    async fn exists(&self, _: &str) -> Result<bool, CacheError> {
        Err(CacheError::OperationFailed("down".into()))
    }
    async fn ping(&self) -> Result<(), CacheError> {
        Err(CacheError::OperationFailed("down".into()))
    }
}

#[tokio::test]
async fn test_a_valkey_that_is_down_never_fails_the_read() {
    let caches = ReadCaches::new(Some(Arc::new(BrokenCache) as Arc<dyn CacheService>));
    let cache = caches.ns(cache_ns::ACL);
    let loads = Loads::default();
    assert_eq!(cache.through("k", &(), 60, || loads.answer(9)).await.unwrap(), 9);
    assert_eq!(cache.through("k", &(), 60, || loads.answer(9)).await.unwrap(), 9);
    assert_eq!(loads.count(), 2, "every read goes to the database");
    assert_eq!(cache.invalidate().await, 0, "and a failed clear is reported, not raised");
}

#[tokio::test]
async fn test_a_disabled_cache_always_loads() {
    let cache = ReadCaches::disabled().ns(cache_ns::REF);
    let loads = Loads::default();
    cache.through("k", &(), 60, || loads.answer(1)).await.unwrap();
    cache.through("k", &(), 60, || loads.answer(1)).await.unwrap();
    assert_eq!(loads.count(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_misses_load_once() {
    let (_, caches) = caches();
    let loads = Loads::default();
    let mut tasks = Vec::new();
    for _ in 0..16 {
        let cache = caches.ns(cache_ns::REF);
        let loads = loads.clone();
        tasks.push(tokio::spawn(async move {
            cache
                .through("slow", &(), 60, || async {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    loads.answer(1).await
                })
                .await
        }));
    }
    for t in tasks {
        assert_eq!(t.await.unwrap().unwrap(), 1);
    }
    assert_eq!(loads.count(), 1, "one database load for sixteen simultaneous readers");
}

#[tokio::test]
async fn test_ttl_has_a_spread_but_stays_near_the_requested_value() {
    let (memory, caches) = caches();
    let cache = caches.ns(cache_ns::REF);
    cache.through("ttl", &(), 1, || async { Ok::<_, String>(1u32) }).await.unwrap();
    // ttl 1 s (spread is 0 for tiny values): the entry is there now and gone after it expires
    assert_eq!(memory.keys("arch:ref:").len(), 1);
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    assert!(memory.keys("arch:ref:").is_empty());
}

#[tokio::test]
async fn test_counters_tell_hits_from_misses_per_namespace() {
    let (_, caches) = caches();
    let loads = Loads::default();
    let refs = caches.ns(cache_ns::REF);
    for _ in 0..4 {
        refs.through("a", &(), 60, || loads.answer(1)).await.unwrap();
    }
    caches.ns(cache_ns::ORG).through("b", &(), 60, || loads.answer(1)).await.unwrap();
    let failing: Result<u32, String> = refs.through("c", &(), 60, || async { Err("db".to_string()) }).await;
    assert!(failing.is_err());

    let counters: std::collections::BTreeMap<_, _> = caches.counters().into_iter().collect();
    assert_eq!((counters["ref"].hits, counters["ref"].misses), (3, 2), "1 miss + 3 hits, and the failed load is a miss too");
    assert_eq!((counters["org"].hits, counters["org"].misses), (0, 1));
    assert!((counters["ref"].hit_ratio() - 60.0).abs() < 0.01);
}

#[tokio::test]
async fn test_a_valkey_that_is_down_is_counted_as_errors() {
    let caches = ReadCaches::new(Some(Arc::new(BrokenCache) as Arc<dyn CacheService>));
    let loads = Loads::default();
    caches.ns(cache_ns::ACL).through("k", &(), 60, || loads.answer(1)).await.unwrap();
    let c = caches.counters().into_iter().find(|(ns, _)| *ns == "acl").unwrap().1;
    assert!(c.errors >= 2, "the failed read and the failed write are both counted, got {}", c.errors);
    assert_eq!((c.hits, c.misses), (0, 1));
}
