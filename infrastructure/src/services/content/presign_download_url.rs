use application_core::errors::ContentError;
use aws_sdk_s3::Client;
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;

/// Presigned GET whose response asks the browser to save the object as `file_name`
pub async fn presign_download_url(
    client: &Client,
    bucket: &str,
    key: &str,
    file_name: &str,
    expires_in_secs: u64,
) -> Result<String, ContentError> {
    let config = PresigningConfig::expires_in(Duration::from_secs(expires_in_secs))
        .map_err(|e| ContentError::PresignError(format!("Invalid presigning duration: {e}")))?;
    // The name goes into a quoted header value: keep it to safe characters
    let safe: String = file_name.chars().filter(|c| c.is_ascii_alphanumeric() || "._- ".contains(*c)).collect();
    let presigned = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .response_content_disposition(format!("attachment; filename=\"{safe}\""))
        .response_content_type("text/csv; charset=utf-8")
        .presigned(config)
        .await
        .map_err(|e| ContentError::PresignError(format!("Failed to presign GET for '{key}': {e}")))?;
    Ok(presigned.uri().to_string())
}
