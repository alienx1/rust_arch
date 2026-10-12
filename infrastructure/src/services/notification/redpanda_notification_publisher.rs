//! `NotificationService` over Redpanda: the notification goes to topic `notifications`, keyed by
//! recipient (one user's notifications stay in order).

use application_core::traits::{
    EventService, NOTIFICATIONS_TOPIC, Notification, NotificationMessage, NotificationService,
};
use async_trait::async_trait;
use std::sync::Arc;

pub struct RedpandaNotificationPublisher {
    events: Arc<dyn EventService>,
}

impl RedpandaNotificationPublisher {
    pub fn new(events: Arc<dyn EventService>) -> Self {
        Self { events }
    }
}

#[async_trait]
impl NotificationService for RedpandaNotificationPublisher {
    async fn notify(&self, username: &str, notification: Notification) {
        let message = NotificationMessage { recipient: username.to_string(), notification };
        let payload = match serde_json::to_vec(&message) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!(error = %e, "notification not serializable");
                return;
            }
        };
        if let Err(e) = self.events.publish(NOTIFICATIONS_TOPIC, Some(username), &payload).await {
            tracing::error!(error = %e, user = username, "notification not published");
        }
    }
}
