//! S3 / Garage Object Storage Content Service implementation.

pub mod delete;
pub mod get;
pub mod get_range;
pub mod multipart_upload;
pub mod presign_download_url;
pub mod presign_get_url;
pub mod put;
pub mod s3_client;

pub use s3_client::s3_client_from_env;

use application_core::errors::ContentError;
use application_core::traits::ContentService;
use async_trait::async_trait;
use aws_sdk_s3::Client;

#[derive(Clone)]
pub struct S3ContentService {
    client: Client,
}

impl S3ContentService {
    /// Create a new S3ContentService with the given aws_sdk_s3::Client.
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Access the underlying AWS S3 client.
    pub fn client(&self) -> &Client {
        &self.client
    }
}

#[async_trait]
impl ContentService for S3ContentService {
    async fn put(
        &self,
        bucket: &str,
        key: &str,
        data: Vec<u8>,
        content_type: Option<&str>,
    ) -> Result<(), ContentError> {
        put::put_content(&self.client, bucket, key, data, content_type).await
    }

    async fn get(&self, bucket: &str, key: &str) -> Result<Vec<u8>, ContentError> {
        get::get_content(&self.client, bucket, key).await
    }

    async fn delete(&self, bucket: &str, key: &str) -> Result<(), ContentError> {
        delete::delete_content(&self.client, bucket, key).await
    }

    async fn presign_get_url(
        &self,
        bucket: &str,
        key: &str,
        expires_in_secs: u64,
    ) -> Result<String, ContentError> {
        presign_get_url::presign_get_url(&self.client, bucket, key, expires_in_secs).await
    }

    async fn get_range(&self, bucket: &str, key: &str, from: u64, to: u64) -> Result<Vec<u8>, ContentError> {
        get_range::get_range(&self.client, bucket, key, from, to).await
    }

    async fn size(&self, bucket: &str, key: &str) -> Result<u64, ContentError> {
        get_range::object_size(&self.client, bucket, key).await
    }

    async fn presign_download_url(
        &self,
        bucket: &str,
        key: &str,
        file_name: &str,
        expires_in_secs: u64,
    ) -> Result<String, ContentError> {
        presign_download_url::presign_download_url(&self.client, bucket, key, file_name, expires_in_secs).await
    }

    async fn begin_multipart(
        &self,
        bucket: &str,
        key: &str,
        content_type: Option<&str>,
    ) -> Result<String, ContentError> {
        multipart_upload::begin(&self.client, bucket, key, content_type).await
    }

    async fn upload_part(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        part_number: i32,
        data: Vec<u8>,
    ) -> Result<String, ContentError> {
        multipart_upload::upload_part(&self.client, bucket, key, upload_id, part_number, data).await
    }

    async fn complete_multipart(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        parts: Vec<(i32, String)>,
    ) -> Result<(), ContentError> {
        multipart_upload::complete(&self.client, bucket, key, upload_id, parts).await
    }

    async fn abort_multipart(&self, bucket: &str, key: &str, upload_id: &str) -> Result<(), ContentError> {
        multipart_upload::abort(&self.client, bucket, key, upload_id).await
    }
}
