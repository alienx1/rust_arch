//! `NotificationPush` over server-sent events: one broadcast channel per user, so every open tab
//! and device of a user gets each event. The HTTP side (an SSE response that reads
//! [`SseNotificationHub::subscribe`]) lives in the API crate.

use application_core::traits::{NotificationPush, PushEvent};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::broadcast;

/// Events a slow tab may fall behind by before it misses some
const CHANNEL_CAPACITY: usize = 64;

#[derive(Default)]
pub struct SseNotificationHub {
    channels: Mutex<HashMap<String, broadcast::Sender<PushEvent>>>,
}

impl SseNotificationHub {
    pub fn new() -> Self {
        Self::default()
    }

    /// Receiver of everything pushed to `username` from now on
    pub fn subscribe(&self, username: &str) -> broadcast::Receiver<PushEvent> {
        let mut channels = self.channels.lock().unwrap_or_else(|e| e.into_inner());
        channels
            .entry(username.to_string())
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0)
            .subscribe()
    }

    /// Ends every open stream (their receivers see the channel close). Called when the process is
    /// shutting down: a server that waits for its connections to finish would otherwise wait for
    /// ever on streams that are meant to stay open.
    pub fn close_all(&self) {
        self.channels.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }
}

#[async_trait]
impl NotificationPush for SseNotificationHub {
    async fn push(&self, username: &str, event: PushEvent) {
        let mut channels = self.channels.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(tx) = channels.get(username) {
            // No receiver left: forget the channel so the map does not grow with every user
            if tx.send(event).is_err() {
                channels.remove(username);
            }
        }
    }
}
