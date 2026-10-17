//! Read-through cache for data that is read often and changes rarely (Valkey behind
//! [`CacheService`]): web -> API -> Valkey -> database.
//!
//! A feature wraps a read in [`ReadCache::through`]: a hit answers from Valkey, a miss runs the
//! database load and stores the result. Writes keep the cache true two ways: the API clears the
//! family of keys a write touches ([`ReadCache::invalidate`], done by the cache-invalidation
//! middleware after the write succeeded) and may refill it at once ([`CacheRefresher`]); the TTL
//! is only a backstop for changes made outside the API (seed scripts, other regions).
//!
//! The cache never fails a request: a Valkey that is down, slow or returning rubbish is skipped
//! and the database answers. Keys are `arch:<namespace>:v1:<language>:<name>:<hash of arguments>`
//! so a namespace is dropped with one prefix delete and localized answers never mix.

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::cache::CacheService;
use crate::request_language;

/// Namespaces: one family of keys per kind of data, dropped together
pub mod cache_ns {
    /// i18n dictionaries (`db_label`) and the API error templates
    pub const I18N: &str = "i18n";
    /// Reference data: geography, lookups, currencies, languages, company / branch types
    pub const REF: &str = "ref";
    /// Organization structure options and summaries (companies, branches, departments, groups)
    pub const ORG: &str = "org";
    /// Program catalogue
    pub const PROGRAMS: &str = "programs";
    /// Sidebar menu per account
    pub const MENU: &str = "menu";
    /// Permission snapshot per account
    pub const ACL: &str = "acl";
    /// Dashboard counters
    pub const DASHBOARD: &str = "dashboard";
    /// Every namespace (a seed run or a manual flush drops them all)
    pub const ALL: [&str; 7] = [I18N, REF, ORG, PROGRAMS, MENU, ACL, DASHBOARD];
}

/// Time to live per kind of data, seconds. The write path clears keys at once; these are the
/// worst case for a change the API did not see. Permissions stay short: a revoked right must not
/// outlive its TTL even if an invalidation is lost.
pub mod cache_ttl {
    pub const I18N: u64 = 60 * 60;
    pub const REF: u64 = 6 * 60 * 60;
    pub const ORG: u64 = 5 * 60;
    pub const PROGRAMS: u64 = 10 * 60;
    pub const MENU: u64 = 60;
    pub const ACL: u64 = 60;
    pub const DASHBOARD: u64 = 60;
}

/// Rebuilds a namespace after it was cleared (called after a write), so the next reader finds the
/// data already in Valkey instead of paying the database round trip
#[async_trait]
pub trait CacheRefresher: Send + Sync {
    async fn refresh(&self, namespace: &str);
}

type Flights = Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>;

/// Log target of every cache line: `RUST_LOG=arch::cache=debug` shows hits and misses,
/// `arch::cache=info` only clears, refills and the periodic summary, `arch::cache=off` nothing
pub const LOG_TARGET: &str = "arch::cache";

/// Reads of one namespace since the process started
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CacheCounters {
    pub hits: u64,
    pub misses: u64,
    /// Valkey calls that failed (the database answered instead)
    pub errors: u64,
}

impl CacheCounters {
    /// Share of reads answered by the cache, 0.0..=100.0
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f64 * 100.0 / total as f64 }
    }
}

type Stats = Arc<Mutex<std::collections::BTreeMap<&'static str, CacheCounters>>>;

/// One namespace of the read cache; cheap to clone
#[derive(Clone)]
pub struct ReadCache {
    cache: Option<Arc<dyn CacheService>>,
    namespace: &'static str,
    /// Keys carry the language of the request (answers that follow `Accept-Language`)
    localized: bool,
    /// Keys also carry what the request may look at (answers narrowed by the view scope)
    scoped: bool,
    stats: Stats,
    /// Loads in progress per key: concurrent misses for one key run the database load once
    flights: Flights,
}

fn elapsed_ms(since: std::time::Instant) -> f64 {
    (since.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0
}

/// The key without the `arch:` prefix and without the version, for readable log lines
fn short(key: &str) -> &str {
    key.strip_prefix("arch:").unwrap_or(key)
}

/// FNV-1a: a stable hash (unlike `DefaultHasher`) so every instance and release builds the same key
fn fnv1a(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// +-10 % so keys written together do not all expire in the same second
fn with_jitter(ttl_secs: u64) -> u64 {
    let spread = ttl_secs / 10;
    if spread == 0 {
        return ttl_secs;
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0) as u64;
    ttl_secs - spread + nanos % (2 * spread + 1)
}

impl ReadCache {
    /// `cache = None` makes every read go to the database (cache off, tests)
    pub fn new(cache: Option<Arc<dyn CacheService>>, namespace: &'static str) -> Self {
        Self { cache, namespace, localized: true, scoped: false, stats: Arc::default(), flights: Arc::default() }
    }

    pub fn disabled(namespace: &'static str) -> Self {
        Self::new(None, namespace)
    }

    /// Same namespace for answers that do not depend on the request language (one entry for
    /// everybody)
    pub fn language_free(mut self) -> Self {
        self.localized = false;
        self
    }

    /// Prefix of every key of this namespace
    pub fn prefix(namespace: &str) -> String {
        format!("arch:{namespace}:")
    }

    /// Key of read `name` with `args` for the language of the current request
    pub fn key(&self, name: &str, args: &impl Serialize) -> String {
        let args = serde_json::to_string(args).unwrap_or_default();
        let language = match (self.localized, request_language::current()) {
            (false, _) => "all",
            (true, request_language::Language::Thai) => "th",
            (true, request_language::Language::English) => "en",
        };
        if self.scoped {
            return format!(
                "{}v1:{language}:{}:{name}:{:016x}",
                Self::prefix(self.namespace),
                crate::view_scope::signature(),
                fnv1a(&args)
            );
        }
        format!("{}v1:{language}:{name}:{:016x}", Self::prefix(self.namespace), fnv1a(&args))
    }

    fn count(&self, update: impl FnOnce(&mut CacheCounters)) {
        let mut stats = self.stats.lock().unwrap_or_else(|e| e.into_inner());
        update(stats.entry(self.namespace).or_default());
    }

    async fn read<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let cache = self.cache.as_ref()?;
        match cache.get(key).await {
            Ok(Some(raw)) => match serde_json::from_str(&raw) {
                Ok(value) => Some(value),
                Err(e) => {
                    // A shape that no longer fits (a release changed the type): treat as a miss
                    tracing::warn!(target: LOG_TARGET, key, error = %e, "cached value unreadable, reloading");
                    None
                }
            },
            Ok(None) => None,
            Err(e) => {
                self.count(|c| c.errors += 1);
                tracing::warn!(target: LOG_TARGET, key, error = %e, "cache read failed, using the database");
                None
            }
        }
    }

    async fn write<T: Serialize>(&self, key: &str, value: &T, ttl_secs: u64) {
        let Some(cache) = self.cache.as_ref() else { return };
        let Ok(raw) = serde_json::to_string(value) else { return };
        if let Err(e) = cache.set(key, &raw, Some(with_jitter(ttl_secs))).await {
            self.count(|c| c.errors += 1);
            tracing::warn!(target: LOG_TARGET, key, error = %e, "cache write failed");
        }
    }

    /// The cached value of read `name` / `args`, or `load()` stored for `ttl_secs`. Errors of
    /// `load` are returned and never cached. When `load` needs to own the arguments, build the key
    /// first with [`key`](Self::key) and call [`through_key`](Self::through_key).
    pub async fn through<T, E, F, Fut>(
        &self,
        name: &str,
        args: &impl Serialize,
        ttl_secs: u64,
        load: F,
    ) -> Result<T, E>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        self.through_key(self.key(name, args), ttl_secs, load).await
    }

    /// [`through`](Self::through) with the key already built
    pub async fn through_key<T, E, F, Fut>(&self, key: String, ttl_secs: u64, load: F) -> Result<T, E>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        if self.cache.is_none() {
            return load().await;
        }
        let started = std::time::Instant::now();
        if let Some(hit) = self.read::<T>(&key).await {
            self.count(|c| c.hits += 1);
            tracing::debug!(target: LOG_TARGET, "HIT   {} ({} ms)", short(&key), elapsed_ms(started));
            return Ok(hit);
        }

        // One load per key at a time: the others wait, then find it in the cache
        let gate = {
            let mut flights = self.flights.lock().unwrap_or_else(|e| e.into_inner());
            flights.entry(key.clone()).or_default().clone()
        };
        let guard = gate.lock().await;
        let result = match self.read::<T>(&key).await {
            Some(hit) => {
                // Another request loaded it while this one waited
                self.count(|c| c.hits += 1);
                tracing::debug!(target: LOG_TARGET, "HIT   {} ({} ms, loaded by a concurrent request)", short(&key), elapsed_ms(started));
                Ok(hit)
            }
            None => {
                self.count(|c| c.misses += 1);
                let loading = std::time::Instant::now();
                let loaded = load().await;
                match &loaded {
                    Ok(value) => {
                        let db_ms = elapsed_ms(loading);
                        self.write(&key, value, ttl_secs).await;
                        tracing::debug!(
                            target: LOG_TARGET,
                            "MISS  {} -> database {db_ms} ms, stored for {ttl_secs} s", short(&key)
                        );
                    }
                    Err(_) => tracing::debug!(
                        target: LOG_TARGET,
                        "MISS  {} -> the load failed, nothing stored", short(&key)
                    ),
                }
                loaded
            }
        };
        drop(guard);
        drop(gate);
        let mut flights = self.flights.lock().unwrap_or_else(|e| e.into_inner());
        if flights.get(&key).is_some_and(|g| Arc::strong_count(g) == 1) {
            flights.remove(&key);
        }
        result
    }

    /// Stores `value` as the answer of read `name` / `args` (the write path refreshes a key it
    /// just changed instead of only dropping it)
    pub async fn put<T: Serialize>(&self, name: &str, args: &impl Serialize, ttl_secs: u64, value: &T) {
        let key = self.key(name, args);
        self.write(&key, value, ttl_secs).await;
        tracing::debug!(target: LOG_TARGET, "PUT   {} (refreshed by a write)", short(&key));
    }

    /// Drops every key of `namespace`; returns how many were removed
    pub async fn invalidate_namespace(cache: &dyn CacheService, namespace: &str) -> u64 {
        match cache.del_prefix(&Self::prefix(namespace)).await {
            Ok(n) => {
                tracing::debug!(target: LOG_TARGET, "CLEAR {namespace}: {n} key(s) removed");
                n
            }
            Err(e) => {
                tracing::warn!(target: LOG_TARGET, namespace, error = %e, "cache clear failed; the TTL will expire the entries");
                0
            }
        }
    }

    /// Drops every key of this namespace
    pub async fn invalidate(&self) -> u64 {
        match self.cache.as_ref() {
            Some(cache) => Self::invalidate_namespace(cache.as_ref(), self.namespace).await,
            None => 0,
        }
    }
}

/// The cache connection shared by a service, handing out one [`ReadCache`] per namespace.
/// Executors hold this and pick the namespace per read; loads in flight are shared across them.
#[derive(Clone)]
pub struct ReadCaches {
    cache: Option<Arc<dyn CacheService>>,
    flights: Flights,
    stats: Stats,
}

impl ReadCaches {
    pub fn new(cache: Option<Arc<dyn CacheService>>) -> Self {
        Self { cache, flights: Arc::default(), stats: Arc::default() }
    }

    /// Every read goes to the database (cache off, tests)
    pub fn disabled() -> Self {
        Self::new(None)
    }

    pub fn is_enabled(&self) -> bool {
        self.cache.is_some()
    }

    /// The cache of one namespace (see [`cache_ns`])
    pub fn ns(&self, namespace: &'static str) -> ReadCache {
        // Organization structure and the dashboard are narrowed by the request's view scope, so
        // their entries are kept per scope
        let scoped = namespace == cache_ns::ORG || namespace == cache_ns::DASHBOARD;
        ReadCache { cache: self.cache.clone(), namespace, localized: true, scoped, stats: self.stats.clone(), flights: self.flights.clone() }
    }

    /// Hits / misses / failures per namespace since start (for the periodic log line and tests)
    pub fn counters(&self) -> Vec<(&'static str, CacheCounters)> {
        self.stats.lock().unwrap_or_else(|e| e.into_inner()).iter().map(|(ns, c)| (*ns, *c)).collect()
    }

    /// The underlying connection (the invalidation middleware needs it)
    pub fn service(&self) -> Option<Arc<dyn CacheService>> {
        self.cache.clone()
    }
}

/// Logs, every `every`, how each namespace is doing (`HIT RATIO 97.5% hits=390 misses=10 ...`),
/// only when something was read since the last line. Runs until the process ends.
pub fn spawn_stats_logger(caches: ReadCaches, every: std::time::Duration) {
    if !caches.is_enabled() || every.is_zero() {
        return;
    }
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(every);
        ticker.tick().await; // the first tick is immediate
        let mut last: Vec<(&'static str, CacheCounters)> = Vec::new();
        loop {
            ticker.tick().await;
            let now = caches.counters();
            if now == last {
                continue;
            }
            for (ns, c) in &now {
                let before = last.iter().find(|(n, _)| n == ns).map(|(_, c)| *c).unwrap_or_default();
                let (hits, misses) = (c.hits - before.hits, c.misses - before.misses);
                if hits + misses + (c.errors - before.errors) == 0 {
                    continue;
                }
                tracing::info!(
                    target: LOG_TARGET,
                    "STATS {ns}: last {}s hits={hits} misses={misses} errors={} ratio={:.1}% | since start hits={} misses={} ratio={:.1}%",
                    every.as_secs(), c.errors - before.errors,
                    CacheCounters { hits, misses, errors: 0 }.hit_ratio(),
                    c.hits, c.misses, c.hit_ratio()
                );
            }
            last = now;
        }
    });
}
