//! SMTP delivery with `lettre`.
//!
//! Configuration lives in `su_configuration` (category `mail`), read on every send so a change
//! takes effect without a restart:
//! - `mail.smtp_url`  `smtp://user:pass@host:1025` (plain, e.g. Mailpit), `smtps://…` (TLS from the
//!   first byte) or `smtp://host:587?tls=required` (STARTTLS); empty = no mail server
//! - `mail.from`      sender, e.g. `Arch Master <no-reply@arch.local>` (default `no-reply@arch.local`)


use application_core::traits::{MailError, MailMessage, MailService, NoMailService};
use async_trait::async_trait;
use lettre::message::{Mailbox, MultiPart, SinglePart, header::ContentType};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use domain::entity::su_configuration;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

pub struct SmtpMailService {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl SmtpMailService {
    pub fn new(smtp_url: &str, from: &str) -> Result<Self, MailError> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::from_url(smtp_url)
            .map_err(|e| MailError::Delivery(format!("bad mail.smtp_url: {e}")))?
            .build();
        let from = from.parse::<Mailbox>().map_err(|_| MailError::BadAddress(from.to_string()))?;
        Ok(Self { transport, from })
    }
}

#[async_trait]
impl MailService for SmtpMailService {
    async fn send(&self, message: MailMessage) -> Result<(), MailError> {
        let to = message.to.parse::<Mailbox>().map_err(|_| MailError::BadAddress(message.to.clone()))?;
        let builder = Message::builder().from(self.from.clone()).to(to).subject(message.subject);
        let email = match message.html {
            Some(html) => builder.multipart(MultiPart::alternative_plain_html(message.text, html)),
            None => builder.singlepart(SinglePart::builder().header(ContentType::TEXT_PLAIN).body(message.text)),
        }
        .map_err(|e| MailError::Delivery(e.to_string()))?;
        self.transport.send(email).await.map(|_| ()).map_err(|e| MailError::Delivery(e.to_string()))
    }
}

/// Mail service whose server comes from `su_configuration`; without `mail.smtp_url` every send is
/// `NotConfigured` (the admin then gets the link to hand over another way)
pub struct ConfiguredMailService {
    db: DatabaseConnection,
}

impl ConfiguredMailService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn setting(&self, key: &str) -> Result<String, MailError> {
        let row = su_configuration::Entity::find()
            .filter(su_configuration::Column::Key.eq(key))
            .one(&self.db)
            .await
            .map_err(|e| MailError::Delivery(format!("cannot read {key}: {e}")))?;
        Ok(row.map(|r| r.value.trim().to_string()).unwrap_or_default())
    }
}

#[async_trait]
impl MailService for ConfiguredMailService {
    async fn send(&self, message: MailMessage) -> Result<(), MailError> {
        let url = self.setting("mail.smtp_url").await?;
        if url.is_empty() {
            return NoMailService.send(message).await;
        }
        let from = match self.setting("mail.from").await? {
            f if f.is_empty() => "no-reply@arch.local".to_string(),
            f => f,
        };
        SmtpMailService::new(&url, &from)?.send(message).await
    }
}
