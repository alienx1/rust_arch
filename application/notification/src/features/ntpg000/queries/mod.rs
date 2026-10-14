//! NTPG000 Queries trait: notification rows

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::ntpg000::errors::Ntpg000Error;
use crate::features::ntpg000::models::NotificationModel;

pub mod db_ntpg000_queries;

pub use db_ntpg000_queries::DbNtpg000Queries;

#[async_trait]
pub trait Ntpg000Queries: Send + Sync {
    /// Stores `row` unless a notification with that id exists; `true` when it was new
    async fn insert_if_absent(&self, row: NotificationModel) -> Result<bool, Ntpg000Error>;

    /// Newest first
    async fn list(&self, recipient: &str, limit: u64) -> Result<Vec<NotificationModel>, Ntpg000Error>;

    /// Deletes everything of `recipient` but the newest `keep` notifications; returns how many
    async fn prune(&self, recipient: &str, keep: u64) -> Result<u64, Ntpg000Error>;

    async fn count_unread(&self, recipient: &str) -> Result<u64, Ntpg000Error>;

    /// Marks one notification of `recipient` as read; `false` when there is no such unread one
    async fn mark_read(&self, recipient: &str, id: Uuid) -> Result<bool, Ntpg000Error>;

    /// Marks every unread notification of `recipient` as read; returns how many
    async fn mark_all_read(&self, recipient: &str) -> Result<u64, Ntpg000Error>;
}
