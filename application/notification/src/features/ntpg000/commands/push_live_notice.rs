//! NTPG000 live action: push a notice to the recipient's open sessions, storing nothing

use application_core::traits::{NotificationMessage, PushEvent};

use super::ntpg000_executor::Ntpg000CommandExecutor;

impl Ntpg000CommandExecutor {
    pub(super) async fn push_live_action(&self, message: NotificationMessage) {
        let recipient = message.recipient.trim().to_string();
        if recipient.is_empty() {
            tracing::error!(id = %message.notification.id, "live notice without a recipient skipped");
            return;
        }
        // The link goes to a web path only, like a stored notification's
        let mut notification = message.notification;
        notification.link = notification
            .link
            .map(|l| l.trim().to_string())
            .filter(|l| l.starts_with('/') && !l.starts_with("//") && !l.contains("://"));
        self.push.push(&recipient, PushEvent::Live(notification)).await;
    }
}
