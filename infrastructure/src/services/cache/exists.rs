use application_core::errors::CacheError;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;

pub async fn exists_cache(conn: &mut MultiplexedConnection, key: &str) -> Result<bool, CacheError> {
    conn.exists(key)
        .await
        .map_err(|e| CacheError::OperationFailed(format!("EXISTS error for key '{key}': {e}")))
}
