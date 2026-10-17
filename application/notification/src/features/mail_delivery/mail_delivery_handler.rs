//! Event handler of `mail-outbox`: sends the queued message through the mail port, retrying a
//! mail server that does not answer.

use std::sync::Arc;
use std::time::Duration;

use application_core::traits::{
    EventHandler, MailError, MailMessage, MailService, Notification, NotificationMessage, notification_kind,
};
use async_trait::async_trait;

use crate::features::ntpg000::Ntpg000Commands;

/// Waits between the attempts: three attempts in all
const DEFAULT_RETRY_DELAYS: [Duration; 2] = [Duration::from_secs(5), Duration::from_secs(30)];

pub struct MailDeliveryHandler {
    mail: Arc<dyn MailService>,
    retry_delays: Vec<Duration>,
    /// Where "your mail could not be sent" goes (the requester's notifications)
    failures: Option<Arc<dyn Ntpg000Commands>>,
}

impl MailDeliveryHandler {
    pub fn new(mail: Arc<dyn MailService>) -> Self {
        Self { mail, retry_delays: DEFAULT_RETRY_DELAYS.to_vec(), failures: None }
    }

    /// Tells the person who asked for a mail, in their notifications, when it could not be delivered
    pub fn with_failure_notices(mut self, notifications: Arc<dyn Ntpg000Commands>) -> Self {
        self.failures = Some(notifications);
        self
    }

    /// Other waits between attempts (tests use none)
    pub fn with_retry_delays(mut self, delays: Vec<Duration>) -> Self {
        self.retry_delays = delays;
        self
    }

    /// Sends `message`; returns the last error when every attempt failed. A mail that can never
    /// be sent (bad address, no mail server set up) is not retried.
    pub async fn deliver(&self, message: MailMessage) -> Result<(), MailError> {
        let mut delays = self.retry_delays.iter();
        loop {
            match self.mail.send(message.clone()).await {
                Ok(()) => {
                    tracing::info!("mail delivered");
                    return Ok(());
                }
                Err(e @ (MailError::BadAddress(_) | MailError::NotConfigured)) => return Err(e),
                Err(e) => match delays.next() {
                    Some(wait) => {
                        tracing::warn!("mail not delivered ({e}), retrying in {} s", wait.as_secs());
                        tokio::time::sleep(*wait).await;
                    }
                    None => return Err(e),
                },
            }
        }
    }
}

impl MailDeliveryHandler {
    /// A notification for whoever asked for `message`; nobody asked = nothing to do
    async fn tell_requester(&self, message: &MailMessage, error: &MailError) {
        let (Some(notifications), Some(requester)) = (&self.failures, message.requested_by.clone()) else { return };
        let to = masked(&message.to);
        let reason = error.to_string();
        let notification = Notification {
            id: uuid::Uuid::now_v7().to_string(),
            kind: notification_kind::MAIL_FAILED.to_string(),
            title: "ส่งอีเมลไม่สำเร็จ".to_string(),
            body: format!("ส่งอีเมลถึง {to} ไม่สำเร็จ: {reason}"),
            data: serde_json::json!({ "to": to, "reason": reason }),
            link: Some("/su/supg004".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        notifications.ingest(NotificationMessage { recipient: requester, notification }).await;
    }
}

/// `s***@arch.local`: the notification shows where it went without spelling the address out
fn masked(address: &str) -> String {
    match address.split_once('@') {
        Some((local, domain)) => format!("{}***@{domain}", local.chars().next().unwrap_or('*')),
        None => "***".to_string(),
    }
}

#[async_trait]
impl EventHandler for MailDeliveryHandler {
    async fn handle(&self, _key: Option<String>, payload: Vec<u8>) {
        match serde_json::from_slice::<MailMessage>(&payload) {
            Ok(message) => {
                if let Err(e) = self.deliver(message.clone()).await {
                    tracing::error!(error = %e, "mail could not be delivered");
                    self.tell_requester(&message, &e).await;
                }
            }
            // A message we cannot read stays unreadable: nothing to retry
            Err(e) => tracing::error!(error = %e, "unreadable mail skipped"),
        }
    }
}
