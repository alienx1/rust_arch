//! Writes a CSV of unknown (possibly huge) size to object storage without holding it in memory:
//! text is buffered and sent as multipart upload parts of [`UPLOAD_PART_BYTES`]; a file that never
//! outgrows one part is a plain PUT.

use application_core::errors::ContentError;
use application_core::traits::ContentService;
use std::sync::Arc;

use crate::features::rppg000::models::UPLOAD_PART_BYTES;

const CONTENT_TYPE: &str = "text/csv; charset=utf-8";

pub struct CsvObjectWriter {
    content: Arc<dyn ContentService>,
    bucket: String,
    key: String,
    buffer: Vec<u8>,
    upload_id: Option<String>,
    parts: Vec<(i32, String)>,
    /// Bytes handed to [`CsvObjectWriter::write`] so far
    pub written: u64,
}

impl CsvObjectWriter {
    pub fn new(content: Arc<dyn ContentService>, bucket: &str, key: &str) -> Self {
        Self {
            content,
            bucket: bucket.to_string(),
            key: key.to_string(),
            buffer: Vec::with_capacity(UPLOAD_PART_BYTES + 64 * 1024),
            upload_id: None,
            parts: Vec::new(),
            written: 0,
        }
    }

    pub async fn write(&mut self, text: &str) -> Result<(), ContentError> {
        self.buffer.extend_from_slice(text.as_bytes());
        self.written += text.len() as u64;
        if self.buffer.len() >= UPLOAD_PART_BYTES {
            self.flush_part().await?;
        }
        Ok(())
    }

    async fn flush_part(&mut self) -> Result<(), ContentError> {
        let upload_id = match &self.upload_id {
            Some(id) => id.clone(),
            None => {
                let id = self.content.begin_multipart(&self.bucket, &self.key, Some(CONTENT_TYPE)).await?;
                self.upload_id = Some(id.clone());
                id
            }
        };
        let number = self.parts.len() as i32 + 1;
        let data = std::mem::take(&mut self.buffer);
        let etag = self.content.upload_part(&self.bucket, &self.key, &upload_id, number, data).await?;
        self.parts.push((number, etag));
        Ok(())
    }

    /// Completes the object
    pub async fn finish(mut self) -> Result<(), ContentError> {
        if self.upload_id.is_none() {
            let data = std::mem::take(&mut self.buffer);
            return self.content.put(&self.bucket, &self.key, data, Some(CONTENT_TYPE)).await;
        }
        if !self.buffer.is_empty() {
            self.flush_part().await?;
        }
        let upload_id = self.upload_id.clone().unwrap_or_default();
        self.content.complete_multipart(&self.bucket, &self.key, &upload_id, std::mem::take(&mut self.parts)).await
    }

    /// Throws away what was uploaded so far (the job failed); best effort
    pub async fn abort(self) {
        if let Some(upload_id) = &self.upload_id {
            if let Err(e) = self.content.abort_multipart(&self.bucket, &self.key, upload_id).await {
                tracing::warn!(error = %e, key = %self.key, "could not abort the upload");
            }
        }
    }
}
