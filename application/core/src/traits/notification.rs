//! Notifications to users of the web app.
//!
//! Any service sends one through [`NotificationService`] (the adapter publishes it to Redpanda,
//! topic [`NOTIFICATIONS_TOPIC`]); the notification service stores it and pushes it to the user's
//! open sessions through [`NotificationPush`] (server-sent events today). Callers only say who
//! and what.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// `kind` values of [`Notification`]
pub mod notification_kind {
    /// A report finished: `data` = `{job_id, file_name, rows}`
    pub const REPORT_READY: &str = "report_ready";
    /// A report failed: `data` = `{job_id, reason}`
    pub const REPORT_FAILED: &str = "report_failed";
    /// A mail the user asked for could not be delivered: `data` = `{to, reason}`
    pub const MAIL_FAILED: &str = "mail_failed";
}

/// Redpanda topic the notification service consumes
pub const NOTIFICATIONS_TOPIC: &str = "notifications";
/// Default consumer group of the notification service. With several instances give each its own
/// group (every instance must see every message to push to its own open streams; storing is
/// idempotent by notification id).
pub const NOTIFICATION_SERVICE_GROUP: &str = "notification-service";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    /// UUIDv7
    pub id: String,
    pub kind: String,
    /// Text in the sender's language: the fallback for clients that cannot translate `kind`
    pub title: String,
    pub body: String,
    /// Kind specific payload: ids the web app needs to open the thing and the values of the text,
    /// so the web app shows it in the user's language (`ntpg000.kind.<kind>.title` / `.body`,
    /// `{name}` placeholders filled from here)
    pub data: Value,
    /// Web path the notification opens when clicked, e.g. `/rp/rppg000`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    /// RFC 3339, UTC
    pub created_at: String,
}

/// What travels on [`NOTIFICATIONS_TOPIC`]: a notification and who it is for
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotificationMessage {
    /// Kratos username of the recipient
    pub recipient: String,
    #[serde(flatten)]
    pub notification: Notification,
}

/// Sending side, used by every service that wants to tell a user something
#[async_trait]
pub trait NotificationService: Send + Sync {
    /// Queues `notification` for the user named `username`. Fire and forget: a broker that does
    /// not answer is logged, never an error for the caller (the job it reports on is done).
    async fn notify(&self, username: &str, notification: Notification);
}

/// What an open stream of a user receives
#[derive(Clone, Debug, PartialEq)]
pub enum PushEvent {
    /// A new notification
    Notification(Notification),
    /// The user's notifications changed (read elsewhere): reload the list
    Changed,
}

/// Delivering side, owned by the notification service: the open sessions (streams) of users
#[async_trait]
pub trait NotificationPush: Send + Sync {
    /// Sends `event` to every open session of `username` (nobody listening is fine)
    async fn push(&self, username: &str, event: PushEvent);
}
