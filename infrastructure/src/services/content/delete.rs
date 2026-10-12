use application_core::errors::ContentError;
use aws_sdk_s3::Client;

pub async fn delete_content(client: &Client, bucket: &str, key: &str) -> Result<(), ContentError> {
    client
        .delete_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| {
            ContentError::DeleteError(format!(
                "Failed to delete object '{key}' from bucket '{bucket}': {e}"
            ))
        })?;

    Ok(())
}
