//! `AuditLogPort` over Quickwit: buffered ingest (see `audit_flusher`), search and get.

use super::audit_flusher::{AuditQueue, QUEUE_CAPACITY, spawn_flusher};
use super::audit_query::{
    EXPORT_SCROLL_KEEP_ALIVE, audit_time_range, build_audit_query, doc_to_event, export_search_body, page_window,
    quote_term,
};
use super::event_stream_query::{build_event_query, event_doc_to_audit, event_index_for};
use super::quickwit_client::QuickwitClient;
use super::quickwit_config::QuickwitConfig;
use application_core::traits::{
    AUDIT_INDEX, AuditError, audit_kind, AuditEvent, AuditExportChunk, AuditLogPort, AuditPage, AuditSearch,
};
use async_trait::async_trait;
use reqwest::StatusCode;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

pub struct QuickwitAuditLog {
    client: Arc<QuickwitClient>,
    queue: AuditQueue,
}

impl QuickwitAuditLog {
    /// Builds the adapter and starts its single background flusher. Never fails and never talks to
    /// Quickwit here (the index is created lazily on the first flush), so it also constructs while
    /// Quickwit is down. Outside a Tokio runtime nothing can be flushed and events are discarded.
    pub fn start(config: QuickwitConfig) -> Arc<Self> {
        let client = Arc::new(QuickwitClient::new(&config.url));
        let (queue, rx) = AuditQueue::new(QUEUE_CAPACITY);
        if tokio::runtime::Handle::try_current().is_ok() {
            spawn_flusher(client.clone(), rx);
        } else {
            tracing::warn!("audit log started outside a Tokio runtime: events will be dropped");
        }
        tracing::info!(url = %config.url, "audit log: Quickwit adapter started");
        Arc::new(Self { client, queue })
    }

    /// Events dropped so far because the queue was full
    pub fn dropped_events(&self) -> u64 {
        self.queue.dropped()
    }

    /// `GET /search`; on 404 (index lost) re-creates the index once and retries
    async fn query(&self, params: &[(&str, String)]) -> Result<Option<Value>, AuditError> {
        self.query_index(AUDIT_INDEX, params).await
    }

    /// `GET /search` on `index`. Only the audit index is re-created when missing; an event index
    /// is owned by `make event-indexes`, so a 404 there is an empty result.
    async fn query_index(&self, index: &str, params: &[(&str, String)]) -> Result<Option<Value>, AuditError> {
        let url = reqwest::Url::parse_with_params(
            &format!("{}/api/v1/{index}/search", self.client.base_url),
            params,
        )
        .map_err(|e| AuditError::Unavailable(format!("invalid Quickwit URL: {e}")))?;
        for attempt in 0..2 {
            let res = self
                .client
                .http
                .get(url.clone())
                .timeout(Duration::from_secs(15))
                .send()
                .await
                .map_err(|e| AuditError::Unavailable(format!("search request failed: {e}")))?;
            let status = res.status();
            if status == StatusCode::NOT_FOUND && index != AUDIT_INDEX {
                return Ok(None);
            }
            if status == StatusCode::NOT_FOUND {
                self.client.mark_index_missing();
                if attempt == 0 {
                    self.client.ensure_index().await.map_err(AuditError::Unavailable)?;
                    continue;
                }
                return Ok(None);
            }
            if status == StatusCode::BAD_REQUEST {
                let detail = res.text().await.unwrap_or_default();
                return Err(AuditError::InvalidQuery(detail.chars().take(300).collect()));
            }
            if !status.is_success() {
                return Err(AuditError::Unavailable(format!("search answered {status}")));
            }
            let body: Value = res
                .json()
                .await
                .map_err(|e| AuditError::Unavailable(format!("unreadable search response: {e}")))?;
            return Ok(Some(body));
        }
        Ok(None)
    }
}

#[async_trait]
impl AuditLogPort for QuickwitAuditLog {
    fn record(&self, events: Vec<AuditEvent>) {
        self.queue.push(events);
    }

    async fn search(&self, filter: AuditSearch) -> Result<AuditPage, AuditError> {
        let range = audit_time_range(&filter)?;
        let (limit, offset) = page_window(&filter);
        // Redpanda events live in their own index: one kind = one index, no merging
        let event_kind = filter.kind.as_deref().map(str::trim).filter(|k| event_index_for(k).is_some());
        let query = match event_kind {
            Some(kind) => build_event_query(kind, &filter),
            None => build_audit_query(&filter),
        };
        let mut params: Vec<(&str, String)> = vec![
            ("query", query),
            ("max_hits", limit.to_string()),
            ("start_offset", offset.to_string()),
        ];
        if let Some(start) = range.start {
            params.push(("start_timestamp", start.to_string()));
        }
        if let Some(end) = range.end {
            params.push(("end_timestamp", end.to_string()));
        }
        // Quickwit's default order is the timestamp field, newest first: no sort_by needed

        let page = filter.page.max(1);
        let index = event_kind.and_then(event_index_for).unwrap_or(AUDIT_INDEX);
        let Some(body) = self.query_index(index, &params).await? else {
            return Ok(AuditPage { items: Vec::new(), total: 0, page, limit });
        };
        let total = body["num_hits"].as_u64().unwrap_or(0);
        let items = body["hits"]
            .as_array()
            .map(|hits| {
                hits.iter()
                    .filter_map(|h| match event_kind {
                        Some(kind) => event_doc_to_audit(kind, h),
                        None => doc_to_event(h.clone()).ok(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(AuditPage { items, total, page, limit })
    }

    async fn get(&self, event_id: &str) -> Result<Option<AuditEvent>, AuditError> {
        let params = [
            ("query", format!("event_id:{}", quote_term(event_id))),
            ("max_hits", "1".to_string()),
        ];
        if let Some(body) = self.query(&params).await? {
            if let Some(hit) = body["hits"].as_array().and_then(|h| h.first()) {
                return doc_to_event(hit.clone()).map(Some);
            }
        }
        // Not an audit event: a Redpanda event (job id / notification id)
        for (kind, id_field) in [(audit_kind::EVENT_REPORT_REQUEST, "job_id"), (audit_kind::EVENT_NOTIFICATION, "id")] {
            let Some(index) = event_index_for(kind) else { continue };
            let params =
                [("query", format!("{id_field}:{}", quote_term(event_id))), ("max_hits", "1".to_string())];
            if let Some(body) = self.query_index(index, &params).await? {
                if let Some(event) = body["hits"].as_array().and_then(|h| h.first()).and_then(|h| event_doc_to_audit(kind, h)) {
                    return Ok(Some(event));
                }
            }
        }
        Ok(None)
    }

    /// Elasticsearch-compatible scroll: the first call opens it, the cursor is Quickwit's
    /// `_scroll_id`. An expired scroll (Quickwit restarted, idle > 2 min) is reported as
    /// unavailable.
    async fn export_chunk(
        &self,
        filter: AuditSearch,
        cursor: Option<String>,
    ) -> Result<AuditExportChunk, AuditError> {
        if filter.kind.as_deref().is_some_and(|k| event_index_for(k.trim()).is_some()) {
            return Err(AuditError::InvalidQuery("Redpanda events cannot be exported".to_string()));
        }
        let request = match &cursor {
            None => {
                self.client.ensure_index().await.map_err(AuditError::Unavailable)?;
                self.client
                    .http
                    .post(format!(
                        "{}/api/v1/_elastic/{AUDIT_INDEX}/_search?scroll={EXPORT_SCROLL_KEEP_ALIVE}",
                        self.client.base_url
                    ))
                    .json(&export_search_body(&filter)?)
            }
            Some(scroll_id) => self
                .client
                .http
                .post(format!("{}/api/v1/_elastic/_search/scroll", self.client.base_url))
                .json(&serde_json::json!({ "scroll": EXPORT_SCROLL_KEEP_ALIVE, "scroll_id": scroll_id })),
        };
        let res = request
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| AuditError::Unavailable(format!("export request failed: {e}")))?;
        let status = res.status();
        if status == StatusCode::BAD_REQUEST {
            let detail = res.text().await.unwrap_or_default();
            return Err(AuditError::InvalidQuery(detail.chars().take(300).collect()));
        }
        if !status.is_success() {
            return Err(AuditError::Unavailable(format!("export answered {status}")));
        }
        let body: Value = res
            .json()
            .await
            .map_err(|e| AuditError::Unavailable(format!("unreadable export response: {e}")))?;
        let items: Vec<AuditEvent> = body["hits"]["hits"]
            .as_array()
            .map(|hits| hits.iter().filter_map(|h| doc_to_event(h["_source"].clone()).ok()).collect())
            .unwrap_or_default();
        let raw_hits = body["hits"]["hits"].as_array().map_or(0, Vec::len);
        // A page without hits ends the scroll
        let cursor = if raw_hits == 0 { None } else { body["_scroll_id"].as_str().map(str::to_string) };
        Ok(AuditExportChunk { items, cursor })
    }

    async fn is_available(&self) -> bool {
        self.client
            .http
            .get(format!("{}/health/livez", self.client.base_url))
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}
