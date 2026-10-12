use application_core::errors::ContentError;
use aws_sdk_s3::Client;

pub async fn get_content(
    client: &Client,
    bucket: &str,
    key: &str,
) -> Result<Vec<u8>, ContentError> {
    let resp = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| {
            ContentError::DownloadError(format!(
                "Failed to get object '{key}' from bucket '{bucket}': {e}"
            ))
        })?;

    let data = resp
        .body
        .collect()
        .await
        .map_err(|e| {
            ContentError::DownloadError(format!("Failed to read body stream for '{key}': {e}"))
        })?
        .into_bytes()
        .to_vec();

    Ok(data)
}
