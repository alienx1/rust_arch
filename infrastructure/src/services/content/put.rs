use application_core::errors::ContentError;
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;

pub async fn put_content(
    client: &Client,
    bucket: &str,
    key: &str,
    data: Vec<u8>,
    content_type: Option<&str>,
) -> Result<(), ContentError> {
    let mut req = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(data));

    if let Some(ct) = content_type {
        req = req.content_type(ct);
    }

    req.send().await.map_err(|e| {
        ContentError::UploadError(format!(
            "Failed to put object '{key}' in bucket '{bucket}': {e}"
        ))
    })?;

    Ok(())
}
