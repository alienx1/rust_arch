use application_core::errors::CacheError;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;

pub async fn get_cache(
    conn: &mut MultiplexedConnection,
    key: &str,
) -> Result<Option<String>, CacheError> {
    conn.get(key)
        .await
        .map_err(|e| CacheError::OperationFailed(format!("GET error for key '{key}': {e}")))
}
