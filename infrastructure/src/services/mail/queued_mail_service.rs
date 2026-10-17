//! `MailService` that queues the message for the notification service instead of delivering it.
//!
//! The message goes to topic `mail-outbox` (keyed by recipient); api-notification reads it and sends
//! it with the SMTP settings in `su_configuration` (see `ConfiguredMailService`), retrying a few
//! times. `send` succeeds once the broker has the message, so a mail server that is down does not
//! make the caller wait; a broker that is down is an error (the caller then hands the link over).

use std::sync::Arc;

use application_core::traits::{EventService, MAIL_TOPIC, MailError, MailMessage, MailService};
use async_trait::async_trait;

pub struct QueuedMailService {
    events: Arc<dyn EventService>,
}

impl QueuedMailService {
    pub fn new(events: Arc<dyn EventService>) -> Self {
        Self { events }
    }
}

#[async_trait]
impl MailService for QueuedMailService {
    async fn send(&self, message: MailMessage) -> Result<(), MailError> {
        if !message.to.contains('@') {
            return Err(MailError::BadAddress(message.to));
        }
        let payload = serde_json::to_vec(&message).map_err(|e| MailError::Delivery(e.to_string()))?;
        self.events
            .publish(MAIL_TOPIC, Some(&message.to), &payload)
            .await
            .map_err(|e| MailError::Delivery(format!("mail queue not available: {e}")))
    }
}
