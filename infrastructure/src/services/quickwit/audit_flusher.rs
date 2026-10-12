//! Bounded event queue and the ONE background task that ships batches to Quickwit.
//!
//! `record` never blocks: the queue is a bounded mpsc channel and a full queue drops the event
//! (warning rate-limited to one per 10 s). The flusher sends a batch when 200 events are buffered
//! or every second. When Quickwit fails, at most the newest `RETRY_KEEP` events are kept for the
//! next attempt (retried after a short back-off) and older ones are dropped. Nothing here panics.

use super::audit_query::event_to_doc;
use super::quickwit_client::QuickwitClient;
use application_core::traits::AuditEvent;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{self, Receiver, Sender};

pub const QUEUE_CAPACITY: usize = 10_000;
pub const BATCH_SIZE: usize = 200;
pub const FLUSH_INTERVAL: Duration = Duration::from_secs(1);
/// Events kept for a retry after a failed flush; older ones are dropped
pub const RETRY_KEEP: usize = 2_000;
const RETRY_BACKOFF: Duration = Duration::from_secs(5);
const WARN_EVERY_SECS: u64 = 10;

/// Producer side of the queue (held by `QuickwitAuditLog`)
pub struct AuditQueue {
    tx: Sender<AuditEvent>,
    dropped: AtomicU64,
    last_warn_secs: AtomicU64,
}

impl AuditQueue {
    pub fn new(capacity: usize) -> (Self, Receiver<AuditEvent>) {
        let (tx, rx) = mpsc::channel(capacity);
        (Self { tx, dropped: AtomicU64::new(0), last_warn_secs: AtomicU64::new(0) }, rx)
    }

    /// Queues the events; whatever does not fit is dropped (counted, warned about rate-limited)
    pub fn push(&self, events: Vec<AuditEvent>) {
        for event in events {
            if self.tx.try_send(event).is_err() {
                self.dropped.fetch_add(1, Ordering::Relaxed);
                self.warn_dropped();
            }
        }
    }

    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    fn warn_dropped(&self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let last = self.last_warn_secs.load(Ordering::Relaxed);
        if now.saturating_sub(last) >= WARN_EVERY_SECS
            && self.last_warn_secs.compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed).is_ok()
        {
            tracing::warn!(
                dropped_total = self.dropped(),
                "audit queue full or closed: events are being dropped"
            );
        }
    }
}

/// NDJSON body for `events` (one document per line)
pub fn to_ndjson(events: &[AuditEvent]) -> String {
    let mut body = String::new();
    for e in events {
        if let Ok(line) = serde_json::to_string(&event_to_doc(e)) {
            body.push_str(&line);
            body.push('\n');
        }
    }
    body
}

/// Starts the flusher task on the current runtime
pub fn spawn_flusher(client: Arc<QuickwitClient>, rx: Receiver<AuditEvent>) {
    tokio::spawn(run_flusher(client, rx));
}

async fn run_flusher(client: Arc<QuickwitClient>, mut rx: Receiver<AuditEvent>) {
    let mut buffer: Vec<AuditEvent> = Vec::new();
    let mut retry_at: Option<Instant> = None;
    let mut ticker = tokio::time::interval(FLUSH_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut warned_failure = false;

    loop {
        // (closed, flush_due): a received event only triggers a flush at the batch size, the
        // interval tick flushes whatever is buffered
        let (closed, flush_due) = tokio::select! {
            received = rx.recv() => match received {
                Some(event) => { buffer.push(event); (false, buffer.len() >= BATCH_SIZE) }
                None => (true, true),
            },
            _ = ticker.tick() => (false, true),
        };

        if buffer.is_empty() {
            if closed {
                return;
            }
            continue;
        }
        if !flush_due {
            continue;
        }
        if !closed && retry_at.is_some_and(|t| Instant::now() < t) {
            // Quickwit is down: keep only the newest events until the back-off is over
            trim_to(&mut buffer, RETRY_KEEP);
            continue;
        }
        match flush(&client, &mut buffer).await {
            Ok(()) => {
                retry_at = None;
                if warned_failure {
                    tracing::info!("audit log delivery to Quickwit recovered");
                    warned_failure = false;
                }
            }
            Err(e) => {
                retry_at = Some(Instant::now() + RETRY_BACKOFF);
                if !warned_failure {
                    tracing::warn!("audit log delivery to Quickwit failed (will retry): {e}");
                    warned_failure = true;
                }
                trim_to(&mut buffer, RETRY_KEEP);
            }
        }
        if closed {
            return;
        }
    }
}

/// Keeps the newest `keep` events
fn trim_to(buffer: &mut Vec<AuditEvent>, keep: usize) {
    if buffer.len() > keep {
        let excess = buffer.len() - keep;
        buffer.drain(..excess);
    }
}

/// Sends the buffer in batches of `BATCH_SIZE`; sent events are removed, a failed batch and the
/// rest stay in the buffer
async fn flush(client: &QuickwitClient, buffer: &mut Vec<AuditEvent>) -> Result<(), String> {
    while !buffer.is_empty() {
        let n = buffer.len().min(BATCH_SIZE);
        client.ingest(to_ndjson(&buffer[..n])).await?;
        buffer.drain(..n);
    }
    Ok(())
}
