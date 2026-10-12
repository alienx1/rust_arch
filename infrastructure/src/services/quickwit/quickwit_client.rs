//! Thin HTTP client for one Quickwit index: shared by the flusher (ingest) and the search side.
//! Tracks whether the index is known to exist and re-creates it after a 404 (self-healing).

use super::index_bootstrap::ensure_index;
use application_core::traits::AUDIT_INDEX;
use reqwest::{Client, StatusCode};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::Mutex;

pub struct QuickwitClient {
    pub http: Client,
    pub base_url: String,
    index_ready: AtomicBool,
    bootstrap: Mutex<()>,
}

impl QuickwitClient {
    pub fn new(base_url: &str) -> Self {
        let http = Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            index_ready: AtomicBool::new(false),
            bootstrap: Mutex::new(()),
        }
    }

    /// Creates the index unless a previous call already confirmed it
    pub async fn ensure_index(&self) -> Result<(), String> {
        if self.index_ready.load(Ordering::Acquire) {
            return Ok(());
        }
        let _guard = self.bootstrap.lock().await;
        if self.index_ready.load(Ordering::Acquire) {
            return Ok(());
        }
        ensure_index(&self.http, &self.base_url, AUDIT_INDEX).await?;
        self.index_ready.store(true, Ordering::Release);
        Ok(())
    }

    /// Forget that the index exists (Quickwit answered 404): the next call re-creates it
    pub fn mark_index_missing(&self) {
        self.index_ready.store(false, Ordering::Release);
    }

    /// `POST /api/v1/audit-logs-v1/ingest?commit=auto` with one NDJSON line per document
    pub async fn ingest(&self, ndjson: String) -> Result<(), String> {
        self.ensure_index().await?;
        let res = self
            .http
            .post(format!("{}/api/v1/{AUDIT_INDEX}/ingest?commit=auto", self.base_url))
            .header("content-type", "application/x-ndjson")
            .body(ndjson)
            .send()
            .await
            .map_err(|e| format!("ingest request failed: {e}"))?;
        let status = res.status();
        if status.is_success() {
            return Ok(());
        }
        if status == StatusCode::NOT_FOUND {
            self.mark_index_missing();
        }
        let detail = res.text().await.unwrap_or_default();
        Err(format!("ingest answered {status}: {}", detail.chars().take(300).collect::<String>()))
    }
}
