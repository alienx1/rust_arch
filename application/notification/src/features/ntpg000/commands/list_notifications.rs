//! NTPG000 list action

use super::ntpg000_executor::Ntpg000CommandExecutor;
use crate::features::ntpg000::errors::Ntpg000Error;
use crate::features::ntpg000::models::{DEFAULT_LIST_LIMIT, MAX_LIST_LIMIT, NotificationListDto};

impl Ntpg000CommandExecutor {
    pub(super) async fn list_action(
        &self,
        username: &str,
        limit: Option<u64>,
    ) -> Result<NotificationListDto, Ntpg000Error> {
        let limit = limit.filter(|l| *l > 0).unwrap_or(DEFAULT_LIST_LIMIT).min(MAX_LIST_LIMIT);
        let items = self.notifications.list(username, limit).await?.into_iter().map(Into::into).collect();
        let unread = self.notifications.count_unread(username).await?;
        Ok(NotificationListDto { items, unread })
    }
}
