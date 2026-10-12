use crate::errors::ContentError;
use async_trait::async_trait;

#[async_trait]
pub trait ContentService: Send + Sync {
    /// Upload an object/file to object storage (S3 / Garage).
    async fn put(
        &self,
        bucket: &str,
        key: &str,
        data: Vec<u8>,
        content_type: Option<&str>,
    ) -> Result<(), ContentError>;

    /// Retrieve an object from object storage.
    async fn get(&self, bucket: &str, key: &str) -> Result<Vec<u8>, ContentError>;

    /// Delete an object from object storage.
    async fn delete(&self, bucket: &str, key: &str) -> Result<(), ContentError>;

    /// Generate a presigned download URL with an expiration duration in seconds.
    async fn presign_get_url(
        &self,
        bucket: &str,
        key: &str,
        expires_in_secs: u64,
    ) -> Result<String, ContentError>;

    /// Bytes `from..=to` of an object (a preview of a big file); shorter when the object ends
    /// earlier.
    async fn get_range(
        &self,
        bucket: &str,
        key: &str,
        from: u64,
        to: u64,
    ) -> Result<Vec<u8>, ContentError>;

    /// Size of an object in bytes.
    async fn size(&self, bucket: &str, key: &str) -> Result<u64, ContentError>;

    /// Presigned download URL that makes the browser save the object as `file_name`.
    async fn presign_download_url(
        &self,
        bucket: &str,
        key: &str,
        file_name: &str,
        expires_in_secs: u64,
    ) -> Result<String, ContentError>;

    /// Starts a multipart upload (files too big to hold in memory); returns the upload id.
    async fn begin_multipart(
        &self,
        bucket: &str,
        key: &str,
        content_type: Option<&str>,
    ) -> Result<String, ContentError>;

    /// Uploads part `part_number` (1-based; every part but the last at least 5 MiB); returns its
    /// ETag.
    async fn upload_part(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        part_number: i32,
        data: Vec<u8>,
    ) -> Result<String, ContentError>;

    /// Finishes the upload from `(part_number, etag)` pairs in order.
    async fn complete_multipart(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        parts: Vec<(i32, String)>,
    ) -> Result<(), ContentError>;

    /// Throws an unfinished upload away.
    async fn abort_multipart(&self, bucket: &str, key: &str, upload_id: &str) -> Result<(), ContentError>;
}
