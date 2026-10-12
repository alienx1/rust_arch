use application_core::errors::CacheError;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;

pub async fn del_cache(conn: &mut MultiplexedConnection, key: &str) -> Result<(), CacheError> {
    conn.del(key)
        .await
        .map_err(|e| CacheError::OperationFailed(format!("DEL error for key '{key}': {e}")))
}
