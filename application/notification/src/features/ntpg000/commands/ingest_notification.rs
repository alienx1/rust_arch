//! NTPG000 ingest action: store a published notification, then push it

use application_core::traits::{NotificationMessage, PushEvent};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::ntpg000_executor::Ntpg000CommandExecutor;
use crate::features::ntpg000::models::{MAX_NOTIFICATIONS_PER_USER, NotificationModel};

const MAX_TITLE: usize = 255;
const MAX_BODY: usize = 2000;
const MAX_LINK: usize = 500;

fn cap(text: &str, max: usize) -> String {
    text.chars().take(max).collect()
}

/// Only a path inside the web app: a link to elsewhere would turn a notification into a phishing
/// redirect
fn safe_link(link: Option<&str>) -> Option<String> {
    link.map(str::trim)
        .filter(|l| l.starts_with('/') && !l.starts_with("//") && !l.contains("://"))
        .map(|l| cap(l, MAX_LINK))
}

impl Ntpg000CommandExecutor {
    pub(super) async fn ingest_action(&self, message: NotificationMessage) {
        let n = message.notification;
        let recipient = message.recipient.trim().to_string();
        let Ok(id) = Uuid::parse_str(&n.id) else {
            tracing::error!(id = %n.id, "notification with a bad id skipped");
            return;
        };
        if recipient.is_empty() {
            tracing::error!(%id, "notification without a recipient skipped");
            return;
        }
        let created_at = DateTime::parse_from_rfc3339(&n.created_at)
            .map(|t| t.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let row = NotificationModel {
            id,
            recipient: recipient.clone(),
            kind: cap(n.kind.trim(), 50),
            title: cap(&n.title, MAX_TITLE),
            body: cap(&n.body, MAX_BODY),
            data: n.data.clone(),
            link: safe_link(n.link.as_deref()),
            created_at: created_at.fixed_offset(),
            read_at: None,
            row_version: 1,
        };
        match self.notifications.insert_if_absent(row.clone()).await {
            Ok(true) => {
                match self.notifications.prune(&recipient, MAX_NOTIFICATIONS_PER_USER).await {
                    Ok(0) => {}
                    Ok(n) => tracing::debug!(%recipient, removed = n, "oldest notifications removed (per-user cap)"),
                    Err(e) => tracing::error!(%recipient, error = %e, "notification cap not enforced"),
                }
                let pushed = crate::features::ntpg000::models::NotificationDto::from(row);
                self.push
                    .push(
                        &recipient,
                        PushEvent::Notification(application_core::traits::Notification {
                            id: pushed.id,
                            kind: pushed.kind,
                            title: pushed.title,
                            body: pushed.body,
                            data: pushed.data,
                            link: pushed.link,
                            created_at: pushed.created_at,
                        }),
                    )
                    .await;
            }
            // Redelivered: already stored and pushed
            Ok(false) => tracing::debug!(%id, "notification already stored"),
            Err(e) => tracing::error!(%id, error = %e, "notification not stored"),
        }
    }
}
