//! Object-storage side of the logo / photo images (organizations, companies, employees): checks
//! the file, stores it under a fresh key, reads it back, forgets objects. The owning executor
//! keeps the key in its own table.

use std::sync::Arc;

use application_core::traits::ContentService;
use tracing::warn;
use uuid::Uuid;

use crate::features::db::dbpg000::models::{
    ImageOwner, MAX_IMAGE_BYTES, StoredImage, content_type_of_key, new_image_key, sniff_image,
};

/// Why an image could not be stored or read
#[derive(Debug)]
pub enum ImageStoreError {
    /// The file is empty, too big or not an accepted image
    Invalid(String),
    /// The object store failed
    Storage(String),
}

#[derive(Clone)]
pub struct ImageObjectStore {
    content: Arc<dyn ContentService>,
    bucket: String,
}

impl ImageObjectStore {
    pub fn new(content: Arc<dyn ContentService>, bucket: impl Into<String>) -> Self {
        Self { content, bucket: bucket.into() }
    }

    /// Validates `bytes` (PNG / JPEG / GIF / WebP by content, at most 2 MB) and stores them under a
    /// new key, which is returned
    pub async fn store(&self, owner: ImageOwner, id: Uuid, bytes: Vec<u8>) -> Result<String, ImageStoreError> {
        if bytes.is_empty() {
            return Err(ImageStoreError::Invalid("Image file is empty".to_string()));
        }
        if bytes.len() > MAX_IMAGE_BYTES {
            return Err(ImageStoreError::Invalid(format!(
                "Image is larger than {} MB",
                MAX_IMAGE_BYTES / (1024 * 1024)
            )));
        }
        let Some((content_type, ext)) = sniff_image(&bytes) else {
            return Err(ImageStoreError::Invalid("Image must be PNG, JPEG, GIF or WebP".to_string()));
        };
        let key = new_image_key(owner, id, ext);
        self.content
            .put(&self.bucket, &key, bytes, Some(content_type))
            .await
            .map_err(|e| ImageStoreError::Storage(format!("Image upload failed: {e}")))?;
        Ok(key)
    }

    pub async fn read(&self, key: &str) -> Result<StoredImage, ImageStoreError> {
        let bytes = self
            .content
            .get(&self.bucket, key)
            .await
            .map_err(|e| ImageStoreError::Storage(format!("Image download failed: {e}")))?;
        Ok(StoredImage { bytes, content_type: content_type_of_key(key) })
    }

    /// Best-effort removal (a failure is logged, never returned)
    pub async fn forget(&self, key: &str) {
        if let Err(e) = self.content.delete(&self.bucket, key).await {
            warn!(key, error = %e, "image object not deleted");
        }
    }
}
