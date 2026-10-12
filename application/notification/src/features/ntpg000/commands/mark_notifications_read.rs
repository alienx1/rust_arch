//! NTPG000 mark-as-read actions

use application_core::traits::PushEvent;
use uuid::Uuid;

use super::ntpg000_executor::Ntpg000CommandExecutor;
use crate::features::ntpg000::errors::Ntpg000Error;

impl Ntpg000CommandExecutor {
    pub(super) async fn mark_read_action(&self, username: &str, id: &str) -> Result<(), Ntpg000Error> {
        let not_found = || Ntpg000Error::NotFound(format!("Notification {id} not found"));
        let uuid = Uuid::parse_str(id.trim()).map_err(|_| not_found())?;
        if self.notifications.mark_read(username, uuid).await? {
            self.push.push(username, PushEvent::Changed).await;
            return Ok(());
        }
        // Not newly read: fine when it is already read, an error when it is not the user's at all
        let mine = self.notifications.list(username, super::super::models::MAX_LIST_LIMIT).await?;
        if mine.iter().any(|n| n.id == uuid) { Ok(()) } else { Err(not_found()) }
    }

    pub(super) async fn mark_all_read_action(&self, username: &str) -> Result<(), Ntpg000Error> {
        if self.notifications.mark_all_read(username).await? > 0 {
            self.push.push(username, PushEvent::Changed).await;
        }
        Ok(())
    }
}
