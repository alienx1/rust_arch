//! A notification row and the list answer

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::NotificationModel;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    pub read: bool,
}

impl From<NotificationModel> for NotificationDto {
    fn from(m: NotificationModel) -> Self {
        Self {
            id: m.id.to_string(),
            kind: m.kind,
            title: m.title,
            body: m.body,
            data: m.data,
            link: m.link,
            created_at: m.created_at.with_timezone(&chrono::Utc).to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            read: m.read_at.is_some(),
        }
    }
}

/// Newest first, plus how many the user has not read (all of them, not only the listed ones)
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NotificationListDto {
    pub items: Vec<NotificationDto>,
    pub unread: u64,
}
