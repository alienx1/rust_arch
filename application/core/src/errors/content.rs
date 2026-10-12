use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContentError {
    #[error("Object not found: {0}")]
    NotFound(String),

    #[error("Failed to upload object: {0}")]
    UploadError(String),

    #[error("Failed to download object: {0}")]
    DownloadError(String),

    #[error("Failed to delete object: {0}")]
    DeleteError(String),

    #[error("Presigned URL generation failed: {0}")]
    PresignError(String),

    #[error("Storage connection error: {0}")]
    ConnectionError(String),
}
