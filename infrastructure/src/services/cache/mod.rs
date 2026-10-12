//! Valkey (Redis-compatible) Cache Service implementation.

pub mod del;
pub mod del_prefix;
pub mod exists;
pub mod get;
pub mod ping;
pub mod set;

use application_core::errors::CacheError;
use application_core::traits::CacheService;
use async_trait::async_trait;
use redis::aio::MultiplexedConnection;

#[derive(Clone)]
pub struct ValkeyCacheService {
    conn: MultiplexedConnection,
}

impl ValkeyCacheService {
    /// Create a new Valkey cache service connected to the given URL (e.g. `redis://valkey:6379`).
    pub async fn new(redis_url: &str) -> Result<Self, CacheError> {
        let client = redis::Client::open(redis_url).map_err(|e| {
            CacheError::ConnectionError(format!("Failed to open Valkey client: {e}"))
        })?;

        let conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                CacheError::ConnectionError(format!("Failed to establish Valkey connection: {e}"))
            })?;

        Ok(Self { conn })
    }

    /// Construct service from an existing `MultiplexedConnection`.
    pub fn from_connection(conn: MultiplexedConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl CacheService for ValkeyCacheService {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        let mut conn = self.conn.clone();
        get::get_cache(&mut conn, key).await
    }

    async fn set(&self, key: &str, value: &str, ttl_secs: Option<u64>) -> Result<(), CacheError> {
        let mut conn = self.conn.clone();
        set::set_cache(&mut conn, key, value, ttl_secs).await
    }

    async fn del(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.conn.clone();
        del::del_cache(&mut conn, key).await
    }

    async fn del_prefix(&self, prefix: &str) -> Result<u64, CacheError> {
        let mut conn = self.conn.clone();
        del_prefix::del_prefix_cache(&mut conn, prefix).await
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let mut conn = self.conn.clone();
        exists::exists_cache(&mut conn, key).await
    }

    async fn ping(&self) -> Result<(), CacheError> {
        let mut conn = self.conn.clone();
        ping::ping_cache(&mut conn).await
    }
}
