use application_core::errors::CacheError;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;

pub async fn set_cache(
    conn: &mut MultiplexedConnection,
    key: &str,
    value: &str,
    ttl_secs: Option<u64>,
) -> Result<(), CacheError> {
    if let Some(ttl) = ttl_secs {
        conn.set_ex(key, value, ttl)
            .await
            .map_err(|e| CacheError::OperationFailed(format!("SET_EX error for key '{key}': {e}")))
    } else {
        conn.set(key, value)
            .await
            .map_err(|e| CacheError::OperationFailed(format!("SET error for key '{key}': {e}")))
    }
}
