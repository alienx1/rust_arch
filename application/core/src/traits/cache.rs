use crate::errors::CacheError;
use async_trait::async_trait;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[async_trait]
pub trait CacheService: Send + Sync {
    /// Retrieve a cached value by key.
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError>;

    /// Set a key-value pair in cache with an optional TTL in seconds.
    async fn set(&self, key: &str, value: &str, ttl_secs: Option<u64>) -> Result<(), CacheError>;

    /// Delete a key from cache.
    async fn del(&self, key: &str) -> Result<(), CacheError>;

    /// Delete every key that starts with `prefix`; returns how many were removed. Used to drop a
    /// whole family of cached reads when its data is written (see [`ReadCache`]). The default
    /// removes nothing, for test doubles.
    async fn del_prefix(&self, _prefix: &str) -> Result<u64, CacheError> {
        Ok(0)
    }

    /// Check if a key exists in cache.
    async fn exists(&self, key: &str) -> Result<bool, CacheError>;

    /// Ping the cache server.
    async fn ping(&self) -> Result<(), CacheError>;

    /// Get and deserialize a JSON-encoded object.
    async fn get_json<T: DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, CacheError>
    where
        Self: Sized,
    {
        match self.get(key).await? {
            Some(raw) => serde_json::from_str(&raw)
                .map(Some)
                .map_err(|e| CacheError::SerializationError(e.to_string())),
            None => Ok(None),
        }
    }

    /// Serialize an object to JSON and store it with optional TTL.
    async fn set_json<T: Serialize + Sync + Send>(
        &self,
        key: &str,
        value: &T,
        ttl_secs: Option<u64>,
    ) -> Result<(), CacheError>
    where
        Self: Sized,
    {
        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;
        self.set(key, &serialized, ttl_secs).await
    }
}
