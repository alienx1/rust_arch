use application_core::errors::CacheError;
use redis::aio::MultiplexedConnection;

pub async fn ping_cache(conn: &mut MultiplexedConnection) -> Result<(), CacheError> {
    let pong: String = redis::cmd("PING")
        .query_async(conn)
        .await
        .map_err(|e| CacheError::ConnectionError(format!("PING failed: {e}")))?;

    if pong == "PONG" {
        Ok(())
    } else {
        Err(CacheError::ConnectionError(format!(
            "Unexpected PING response: {pong}"
        )))
    }
}
