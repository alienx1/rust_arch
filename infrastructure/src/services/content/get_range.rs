use application_core::errors::ContentError;
use aws_sdk_s3::Client;

/// Bytes `from..=to` of an object
pub async fn get_range(
    client: &Client,
    bucket: &str,
    key: &str,
    from: u64,
    to: u64,
) -> Result<Vec<u8>, ContentError> {
    let resp = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .range(format!("bytes={from}-{to}"))
        .send()
        .await
        .map_err(|e| {
            if e.as_service_error().is_some_and(|s| s.is_no_such_key()) {
                ContentError::NotFound(format!("'{key}' in bucket '{bucket}'"))
            } else {
                ContentError::DownloadError(format!("Failed to read '{key}' range {from}-{to}: {e}"))
            }
        })?;
    let data = resp
        .body
        .collect()
        .await
        .map_err(|e| ContentError::DownloadError(format!("Failed to read body stream for '{key}': {e}")))?;
    Ok(data.into_bytes().to_vec())
}

/// Size of an object (HEAD)
pub async fn object_size(client: &Client, bucket: &str, key: &str) -> Result<u64, ContentError> {
    let head = client
        .head_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| {
            if e.as_service_error().is_some_and(|s| s.is_not_found()) {
                ContentError::NotFound(format!("'{key}' in bucket '{bucket}'"))
            } else {
                ContentError::DownloadError(format!("Failed to look up '{key}': {e}"))
            }
        })?;
    Ok(head.content_length().unwrap_or(0).max(0) as u64)
}
