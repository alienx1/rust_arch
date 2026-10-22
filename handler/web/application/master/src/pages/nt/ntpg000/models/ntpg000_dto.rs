//! NTPG000 DTOs: a notification and the list answer

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NotificationDto {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub data: Value,
    /// Web path the notification opens
    pub link: Option<String>,
    /// RFC 3339, UTC
    pub created_at: String,
    /// Absent on a notification just pushed over the stream (it is unread by definition)
    #[serde(default)]
    pub read: bool,
}

impl NotificationDto {
    /// Whether the notification reports something that went wrong (shown as an error toast)
    pub fn is_failure(&self) -> bool {
        matches!(self.kind.as_str(), "data_failed" | "report_failed" | "mail_failed")
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NotificationListDto {
    pub items: Vec<NotificationDto>,
    /// Unread in all, not only among `items`
    pub unread: u64,
}
