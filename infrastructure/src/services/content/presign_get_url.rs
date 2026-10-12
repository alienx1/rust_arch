use application_core::errors::ContentError;
use aws_sdk_s3::Client;
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;

pub async fn presign_get_url(
    client: &Client,
    bucket: &str,
    key: &str,
    expires_in_secs: u64,
) -> Result<String, ContentError> {
    let presign_config = PresigningConfig::expires_in(Duration::from_secs(expires_in_secs))
        .map_err(|e| ContentError::PresignError(format!("Invalid presigning duration: {e}")))?;

    let presigned_req = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .presigned(presign_config)
        .await
        .map_err(|e| {
            ContentError::PresignError(format!("Failed to presign GET for object '{key}': {e}"))
        })?;

    Ok(presigned_req.uri().to_string())
}
