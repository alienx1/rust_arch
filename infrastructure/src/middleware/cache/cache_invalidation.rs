//! Keeps the Valkey read cache true when data is written.
//!
//! After a write request (POST / PUT / PATCH / DELETE) has been handled, the namespaces its path
//! belongs to are dropped from Valkey and, where a [`CacheRefresher`] exists, refilled right away,
//! so the next reader gets the new data from the cache instead of the stale copy. It runs after
//! the handler returned, i.e. after the database transaction committed. A write that was refused
//! (4xx) changed nothing and clears nothing; one that failed on the server (5xx) may have changed
//! something, so it clears too.
//!
//! The path -> namespace table is the API's own knowledge of which program feeds which cached
//! reads; see `handler/api/master/src/cache_policy.rs`.

use application_core::traits::{CacheRefresher, CacheService, ReadCache};
use axum::extract::{Request, State};
use axum::http::Method;
use axum::middleware::Next;
use axum::response::Response;
use std::collections::BTreeSet;
use std::sync::Arc;

/// Writes below `path` (a path segment prefix such as `/db/dbpg004`) clear `namespaces`
#[derive(Clone, Copy, Debug)]
pub struct InvalidationRule {
    pub path: &'static str,
    pub namespaces: &'static [&'static str],
}

#[derive(Clone)]
pub struct CacheInvalidation {
    cache: Arc<dyn CacheService>,
    rules: Arc<Vec<InvalidationRule>>,
    refresher: Option<Arc<dyn CacheRefresher>>,
}

impl CacheInvalidation {
    pub fn new(cache: Arc<dyn CacheService>, rules: Vec<InvalidationRule>) -> Self {
        Self { cache, rules: Arc::new(rules), refresher: None }
    }

    /// Refills a cleared namespace in the background (see [`CacheRefresher`])
    pub fn with_refresher(mut self, refresher: Arc<dyn CacheRefresher>) -> Self {
        self.refresher = Some(refresher);
        self
    }
}

/// `true` when `path` is `rule` or below it, matching whole segments (`/db/dbpg00` does not match
/// `/db/dbpg004`). Any prefix before the rule (`/api/v1`, `/api/v1/master`) is allowed.
fn path_matches(path: &str, rule: &str) -> bool {
    let mut from = 0;
    while let Some(found) = path[from..].find(rule) {
        let start = from + found;
        let end = start + rule.len();
        let before_ok = start == 0 || path.as_bytes()[start - 1] == b'/' || rule.starts_with('/');
        let after_ok = end == path.len() || path.as_bytes()[end] == b'/';
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
    }
    false
}

/// Namespaces a write on `method` / `path` clears; empty for reads and unlisted paths
pub fn namespaces_for(rules: &[InvalidationRule], method: &Method, path: &str) -> BTreeSet<&'static str> {
    let is_write = matches!(*method, Method::POST | Method::PUT | Method::PATCH | Method::DELETE);
    if !is_write {
        return BTreeSet::new();
    }
    rules
        .iter()
        .filter(|r| path_matches(path, r.path))
        .flat_map(|r| r.namespaces.iter().copied())
        .collect()
}

/// Axum middleware (`from_fn_with_state`): clears and refills the cache namespaces a successful
/// write touched. The state is `None` when the cache is off: requests pass through untouched.
pub async fn invalidate_cache_on_write(
    State(state): State<Option<CacheInvalidation>>,
    request: Request,
    next: Next,
) -> Response {
    let Some(state) = state else { return next.run(request).await };
    let namespaces = namespaces_for(&state.rules, request.method(), request.uri().path());
    let (method, path) = (request.method().clone(), request.uri().path().to_string());
    let response = next.run(request).await;
    let status = response.status();
    if namespaces.is_empty() || !(status.is_success() || status.is_server_error()) {
        return response;
    }
    let mut report = Vec::new();
    for namespace in &namespaces {
        let removed = ReadCache::invalidate_namespace(state.cache.as_ref(), namespace).await;
        report.push(format!("{namespace}={removed}"));
    }
    tracing::info!(
        target: "arch::cache",
        "WRITE {method} {path} -> {} : cleared {}{}",
        status.as_u16(),
        report.join(" "),
        if state.refresher.is_some() { " (refilling in the background)" } else { "" }
    );
    if let Some(refresher) = &state.refresher {
        for namespace in namespaces {
            let refresher = refresher.clone();
            tokio::spawn(async move { refresher.refresh(namespace).await });
        }
    }
    response
}
