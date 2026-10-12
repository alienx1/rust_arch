use application_core::errors::CacheError;
use redis::aio::MultiplexedConnection;

/// Keys scanned per round trip
const SCAN_BATCH: usize = 500;

/// Deletes every key that starts with `prefix` (SCAN + UNLINK: never blocks the server like KEYS
/// would). Returns how many keys were removed.
pub async fn del_prefix_cache(conn: &mut MultiplexedConnection, prefix: &str) -> Result<u64, CacheError> {
    // `prefix` is a literal: glob metacharacters are escaped so it cannot widen the match
    let mut pattern = String::with_capacity(prefix.len() + 1);
    for c in prefix.chars() {
        if matches!(c, '*' | '?' | '[' | ']' | '\\') {
            pattern.push('\\');
        }
        pattern.push(c);
    }
    pattern.push('*');

    let fail = |e: redis::RedisError| CacheError::OperationFailed(format!("DEL prefix '{prefix}' error: {e}"));
    let (mut cursor, mut removed) = (0u64, 0u64);
    loop {
        let (next, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(&pattern)
            .arg("COUNT")
            .arg(SCAN_BATCH)
            .query_async(conn)
            .await
            .map_err(fail)?;
        if !keys.is_empty() {
            let n: u64 = redis::cmd("UNLINK").arg(&keys).query_async(conn).await.map_err(fail)?;
            removed += n;
        }
        if next == 0 {
            return Ok(removed);
        }
        cursor = next;
    }
}
