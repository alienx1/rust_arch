//! SMTP adapter of the mail port.

mod queued_mail_service;
mod smtp_mail_service;

pub use queued_mail_service::QueuedMailService;
pub use smtp_mail_service::{SmtpMailService, ConfiguredMailService};
