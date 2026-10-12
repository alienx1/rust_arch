//! Tests for the cache-invalidation middleware: which writes clear which namespaces, and that
//! reads and refused writes clear nothing. Located at `infrastructure/tests/cache_invalidation_test.rs`.

use application_core::traits::{CacheRefresher, CacheService, InMemoryCache};
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::routing::{any, get, post};
use axum::{Router, middleware::from_fn_with_state};
use infrastructure::middleware::{
    CacheInvalidation, InvalidationRule, invalidate_cache_on_write, namespaces_for,
};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

const RULES: &[InvalidationRule] = &[
    InvalidationRule { path: "/db/dbpg004", namespaces: &["ref", "dashboard"] },
    InvalidationRule { path: "/db/dbpg006", namespaces: &["i18n"] },
    InvalidationRule { path: "/su/supg003", namespaces: &["acl", "menu"] },
];

fn rules() -> Vec<InvalidationRule> {
    RULES.to_vec()
}

#[test]
fn test_only_writes_on_listed_paths_clear_namespaces() {
    let hit = namespaces_for(&rules(), &Method::PUT, "/api/v1/db/dbpg004/lookups/1");
    assert_eq!(hit.into_iter().collect::<Vec<_>>(), vec!["dashboard", "ref"]);
    for method in [Method::POST, Method::PUT, Method::PATCH, Method::DELETE] {
        assert!(!namespaces_for(&rules(), &method, "/api/v1/master/su/supg003/groups/1/matrix").is_empty());
    }
    assert!(namespaces_for(&rules(), &Method::GET, "/api/v1/db/dbpg004/lookups").is_empty(), "reads clear nothing");
    assert!(namespaces_for(&rules(), &Method::POST, "/api/v1/al/alpg000/exports").is_empty(), "unlisted path");
}

#[test]
fn test_paths_match_whole_segments() {
    assert!(namespaces_for(&rules(), &Method::POST, "/db/dbpg0041/lookups").is_empty());
    assert!(!namespaces_for(&rules(), &Method::POST, "/db/dbpg004").is_empty(), "the program root itself");
}

#[derive(Default)]
struct Refills(Mutex<Vec<String>>);

#[async_trait]
impl CacheRefresher for Refills {
    async fn refresh(&self, namespace: &str) {
        self.0.lock().unwrap().push(namespace.to_string());
    }
}

async fn seeded() -> Arc<InMemoryCache> {
    let memory = Arc::new(InMemoryCache::new());
    for key in ["arch:ref:v1:th:a:1", "arch:dashboard:v1:th:s:1", "arch:i18n:v1:all:d:1", "arch:acl:v1:all:s:1", "other:key"] {
        memory.set(key, "x", Some(60)).await.unwrap();
    }
    memory
}

fn app(memory: Arc<InMemoryCache>, refills: Option<Arc<Refills>>) -> Router {
    let mut state = CacheInvalidation::new(memory as Arc<dyn CacheService>, rules());
    if let Some(r) = refills {
        state = state.with_refresher(r);
    }
    Router::new()
        .route("/db/dbpg004/lookups", post(|| async { StatusCode::CREATED }).get(|| async { "list" }))
        .route("/db/dbpg004/refused", post(|| async { StatusCode::UNPROCESSABLE_ENTITY }))
        .route("/db/dbpg004/broken", post(|| async { StatusCode::INTERNAL_SERVER_ERROR }))
        .route("/db/dbpg006/labels", any(|| async { StatusCode::OK }))
        .layer(from_fn_with_state(Some(state), invalidate_cache_on_write))
}

fn req(method: &str, uri: &str) -> Request<Body> {
    Request::builder().method(method).uri(uri).body(Body::empty()).unwrap()
}

#[tokio::test]
async fn test_a_successful_write_clears_its_namespaces_and_nothing_else() {
    let memory = seeded().await;
    let resp = app(memory.clone(), None).oneshot(req("POST", "/db/dbpg004/lookups")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    assert!(memory.keys("arch:ref:").is_empty());
    assert!(memory.keys("arch:dashboard:").is_empty());
    assert_eq!(memory.keys("arch:i18n:").len(), 1, "another program's data stays");
    assert_eq!(memory.keys("arch:acl:").len(), 1);
    assert_eq!(memory.keys("other:").len(), 1, "keys outside the cache namespaces are never touched");
}

#[tokio::test]
async fn test_reads_and_refused_writes_clear_nothing() {
    let memory = seeded().await;
    let router = app(memory.clone(), None);
    router.clone().oneshot(req("GET", "/db/dbpg004/lookups")).await.unwrap();
    let refused = router.oneshot(req("POST", "/db/dbpg004/refused")).await.unwrap();
    assert_eq!(refused.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(memory.len(), 5);
}

#[tokio::test]
async fn test_a_server_error_may_have_changed_data_so_it_clears_too() {
    let memory = seeded().await;
    app(memory.clone(), None).oneshot(req("POST", "/db/dbpg004/broken")).await.unwrap();
    assert!(memory.keys("arch:ref:").is_empty());
}

#[tokio::test]
async fn test_the_refresher_refills_each_cleared_namespace() {
    let memory = seeded().await;
    let refills = Arc::new(Refills::default());
    app(memory, Some(refills.clone())).oneshot(req("PUT", "/db/dbpg006/labels")).await.unwrap();
    // the refill runs in the background
    for _ in 0..50 {
        if !refills.0.lock().unwrap().is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_eq!(*refills.0.lock().unwrap(), vec!["i18n".to_string()]);
}

#[tokio::test]
async fn test_without_a_cache_requests_pass_through() {
    let router = Router::new()
        .route("/db/dbpg004/lookups", get(|| async { "ok" }))
        .layer(from_fn_with_state(Option::<CacheInvalidation>::None, invalidate_cache_on_write));
    let resp = router.oneshot(req("GET", "/db/dbpg004/lookups")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
