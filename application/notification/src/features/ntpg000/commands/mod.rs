//! NTPG000 Commands trait (the consumer's ingest + what the user reads and changes) and executor

use application_core::traits::NotificationMessage;
use async_trait::async_trait;

use crate::features::ntpg000::errors::Ntpg000Error;
use crate::features::ntpg000::models::NotificationListDto;

mod ingest_notification;
mod list_notifications;
mod mark_notifications_read;
mod notification_message_handler;
pub mod ntpg000_executor;

pub use notification_message_handler::NotificationMessageHandler;
pub use ntpg000_executor::Ntpg000CommandExecutor;

#[async_trait]
pub trait Ntpg000Commands: Send + Sync {
    /// Stores a notification published by some service and pushes it to the recipient's open
    /// sessions. A redelivered message (same id) is stored and pushed once. Never fails the
    /// consumer: a message that cannot be stored is logged and dropped.
    async fn ingest(&self, message: NotificationMessage);

    /// The user's notifications, newest first (`limit` default 20, at most 300 = the per-user cap), and how many are
    /// unread in all
    async fn list(&self, username: &str, limit: Option<u64>) -> Result<NotificationListDto, Ntpg000Error>;

    /// Marks one notification of the user as read (already read is fine; somebody else's or a
    /// missing one is `NotFound`) and tells the user's other sessions to reload
    async fn mark_read(&self, username: &str, id: &str) -> Result<(), Ntpg000Error>;

    /// Marks all of the user's notifications as read and tells the user's other sessions
    async fn mark_all_read(&self, username: &str) -> Result<(), Ntpg000Error>;
}
