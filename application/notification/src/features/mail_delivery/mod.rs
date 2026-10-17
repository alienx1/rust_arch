//! Mail delivery: reads the messages services queued on topic `mail-outbox` and sends them.

mod mail_delivery_handler;

pub use mail_delivery_handler::MailDeliveryHandler;
