//! Creates the `audit-logs-v1` index when it does not exist.
//!
//! Quickwit data may live on an emptyDir, so the index can vanish at any pod restart: callers
//! re-run [`ensure_index`] whenever ingest / search answers 404.

use reqwest::{Client, StatusCode};
use std::time::Duration;

/// The intended doc mapping, embedded in the binary
pub const AUDIT_INDEX_CONFIG: &str = include_str!("audit_logs_v1_index.json");

/// `GET /api/v1/indexes/audit-logs-v1`, then `POST /api/v1/indexes` when it is missing
pub async fn ensure_index(client: &Client, base_url: &str, index: &str) -> Result<(), String> {
    let get = client
        .get(format!("{base_url}/api/v1/indexes/{index}"))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| format!("index lookup failed: {e}"))?;
    match get.status() {
        s if s.is_success() => return Ok(()),
        StatusCode::NOT_FOUND => {}
        s => return Err(format!("index lookup answered {s}")),
    }

    let create = client
        .post(format!("{base_url}/api/v1/indexes"))
        .timeout(Duration::from_secs(10))
        .header("content-type", "application/json")
        .body(AUDIT_INDEX_CONFIG)
        .send()
        .await
        .map_err(|e| format!("index creation failed: {e}"))?;
    let status = create.status();
    if status.is_success() {
        tracing::info!("created Quickwit index `{index}`");
        return Ok(());
    }
    let detail = create.text().await.unwrap_or_default();
    // Created concurrently by another replica: fine
    if status == StatusCode::BAD_REQUEST && detail.contains("already exists") {
        return Ok(());
    }
    Err(format!("index creation answered {status}: {detail}"))
}
