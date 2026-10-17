//! Outgoing e-mail port. The application says who gets what; the adapter (SMTP) delivers it.
//! A deployment without a mail server uses an adapter that always answers
//! [`MailError::NotConfigured`], so callers can tell the admin to hand the link over another way.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Redpanda topic the notification service reads mail from: services queue a [`MailMessage`] there
/// instead of talking to a mail server themselves
pub const MAIL_TOPIC: &str = "mail-outbox";
/// Consumer group of the mail delivery of the notification service
pub const MAIL_SERVICE_GROUP: &str = "mail-delivery-service";

/// One message to one recipient
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailMessage {
    pub to: String,
    pub subject: String,
    /// Plain-text body (always present)
    pub text: String,
    /// HTML alternative, when there is one
    pub html: Option<String>,
    /// Username of the person who asked for this mail: told in their notifications when it cannot
    /// be delivered (`None` = nobody to tell)
    #[serde(default)]
    pub requested_by: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MailError {
    /// No mail server is configured (`mail.smtp_url`)
    NotConfigured,
    /// The recipient or sender address is not valid
    BadAddress(String),
    /// The mail server refused or could not be reached
    Delivery(String),
}

impl std::fmt::Display for MailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotConfigured => write!(f, "no mail server is configured (set mail.smtp_url in su_configuration)"),
            Self::BadAddress(a) => write!(f, "invalid e-mail address: {a}"),
            Self::Delivery(m) => write!(f, "mail delivery failed: {m}"),
        }
    }
}

impl std::error::Error for MailError {}

#[async_trait]
pub trait MailService: Send + Sync {
    async fn send(&self, message: MailMessage) -> Result<(), MailError>;
}

/// Adapter for deployments without a mail server: every send is `NotConfigured`
#[derive(Clone, Copy, Debug, Default)]
pub struct NoMailService;

#[async_trait]
impl MailService for NoMailService {
    async fn send(&self, _message: MailMessage) -> Result<(), MailError> {
        Err(MailError::NotConfigured)
    }
}
