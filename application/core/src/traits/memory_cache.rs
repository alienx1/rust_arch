//! `CacheService` kept in this process's memory: for tests and for running without Valkey. Honours
//! TTLs and prefix deletes like the real thing; it is not shared between processes.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use async_trait::async_trait;

use super::cache::CacheService;
use crate::errors::CacheError;

#[derive(Default)]
pub struct InMemoryCache {
    entries: Mutex<HashMap<String, (String, Option<Instant>)>>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Live keys (expired ones are not counted)
    pub fn len(&self) -> usize {
        let now = Instant::now();
        self.lock().values().filter(|(_, until)| until.is_none_or(|t| t > now)).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Live keys starting with `prefix`, sorted
    pub fn keys(&self, prefix: &str) -> Vec<String> {
        let now = Instant::now();
        let mut keys: Vec<String> = self
            .lock()
            .iter()
            .filter(|(k, (_, until))| k.starts_with(prefix) && until.is_none_or(|t| t > now))
            .map(|(k, _)| k.clone())
            .collect();
        keys.sort();
        keys
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, (String, Option<Instant>)>> {
        self.entries.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[async_trait]
impl CacheService for InMemoryCache {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        let now = Instant::now();
        Ok(self.lock().get(key).filter(|(_, until)| until.is_none_or(|t| t > now)).map(|(v, _)| v.clone()))
    }

    async fn set(&self, key: &str, value: &str, ttl_secs: Option<u64>) -> Result<(), CacheError> {
        let until = ttl_secs.map(|s| Instant::now() + Duration::from_secs(s));
        self.lock().insert(key.to_string(), (value.to_string(), until));
        Ok(())
    }

    async fn del(&self, key: &str) -> Result<(), CacheError> {
        self.lock().remove(key);
        Ok(())
    }

    async fn del_prefix(&self, prefix: &str) -> Result<u64, CacheError> {
        let mut entries = self.lock();
        let before = entries.len();
        entries.retain(|k, _| !k.starts_with(prefix));
        Ok((before - entries.len()) as u64)
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        Ok(self.get(key).await?.is_some())
    }

    async fn ping(&self) -> Result<(), CacheError> {
        Ok(())
    }
}
