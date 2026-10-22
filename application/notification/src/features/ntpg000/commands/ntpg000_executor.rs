//! NTPG000 Command Executor: wires the ports (notification rows, the open sessions) to the
//! actions, each of which lives in its own file.

use application_core::traits::{NotificationMessage, NotificationPush};
use async_trait::async_trait;
use std::sync::Arc;

use super::Ntpg000Commands;
use crate::features::ntpg000::errors::Ntpg000Error;
use crate::features::ntpg000::models::NotificationListDto;
use crate::features::ntpg000::queries::Ntpg000Queries;

pub struct Ntpg000CommandExecutor {
    pub(super) notifications: Arc<dyn Ntpg000Queries>,
    pub(super) push: Arc<dyn NotificationPush>,
}

impl Ntpg000CommandExecutor {
    pub fn new(notifications: Arc<dyn Ntpg000Queries>, push: Arc<dyn NotificationPush>) -> Self {
        Self { notifications, push }
    }
}

#[async_trait]
impl Ntpg000Commands for Ntpg000CommandExecutor {
    async fn ingest(&self, message: NotificationMessage) {
        self.ingest_action(message).await
    }

    async fn push_live(&self, message: NotificationMessage) {
        self.push_live_action(message).await
    }

    async fn list(&self, username: &str, limit: Option<u64>) -> Result<NotificationListDto, Ntpg000Error> {
        self.list_action(username, limit).await
    }

    async fn mark_read(&self, username: &str, id: &str) -> Result<(), Ntpg000Error> {
        self.mark_read_action(username, id).await
    }

    async fn mark_all_read(&self, username: &str) -> Result<(), Ntpg000Error> {
        self.mark_all_read_action(username).await
    }
}
