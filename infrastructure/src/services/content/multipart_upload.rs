use application_core::errors::ContentError;
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};

pub async fn begin(client: &Client, bucket: &str, key: &str, content_type: Option<&str>) -> Result<String, ContentError> {
    let mut req = client.create_multipart_upload().bucket(bucket).key(key);
    if let Some(ct) = content_type {
        req = req.content_type(ct);
    }
    let out = req
        .send()
        .await
        .map_err(|e| ContentError::UploadError(format!("Failed to start upload of '{key}': {e}")))?;
    out.upload_id()
        .map(str::to_string)
        .ok_or_else(|| ContentError::UploadError(format!("No upload id for '{key}'")))
}

pub async fn upload_part(
    client: &Client,
    bucket: &str,
    key: &str,
    upload_id: &str,
    part_number: i32,
    data: Vec<u8>,
) -> Result<String, ContentError> {
    let out = client
        .upload_part()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .part_number(part_number)
        .body(ByteStream::from(data))
        .send()
        .await
        .map_err(|e| ContentError::UploadError(format!("Failed to upload part {part_number} of '{key}': {e}")))?;
    out.e_tag()
        .map(str::to_string)
        .ok_or_else(|| ContentError::UploadError(format!("No ETag for part {part_number} of '{key}'")))
}

pub async fn complete(
    client: &Client,
    bucket: &str,
    key: &str,
    upload_id: &str,
    parts: Vec<(i32, String)>,
) -> Result<(), ContentError> {
    let parts = parts
        .into_iter()
        .map(|(n, etag)| CompletedPart::builder().part_number(n).e_tag(etag).build())
        .collect::<Vec<_>>();
    client
        .complete_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .multipart_upload(CompletedMultipartUpload::builder().set_parts(Some(parts)).build())
        .send()
        .await
        .map_err(|e| ContentError::UploadError(format!("Failed to complete upload of '{key}': {e}")))?;
    Ok(())
}

pub async fn abort(client: &Client, bucket: &str, key: &str, upload_id: &str) -> Result<(), ContentError> {
    client
        .abort_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .send()
        .await
        .map_err(|e| ContentError::UploadError(format!("Failed to abort upload of '{key}': {e}")))?;
    Ok(())
}
