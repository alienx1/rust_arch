//! SUPG006 executor: routes each `Supg006Commands` method to its action file

use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use super::Supg006Commands;
use crate::features::su::supg006::errors::Supg006Error;
use crate::features::su::supg006::models::{ConfigItemDto, SaveConfigCommand};
use crate::features::su::supg006::queries::Supg006Queries;

pub struct Supg006CommandExecutor {
    pub(super) queries: Arc<dyn Supg006Queries>,
    pub(super) db: DatabaseConnection,
}

impl Supg006CommandExecutor {
    pub fn new(queries: Arc<dyn Supg006Queries>, db: DatabaseConnection) -> Self {
        Self { queries, db }
    }
}

#[async_trait]
impl Supg006Commands for Supg006CommandExecutor {
    async fn list_config(&self) -> Result<Vec<ConfigItemDto>, Supg006Error> {
        self.list_config().await
    }

    async fn save_config(&self, cmd: SaveConfigCommand, actor_id: Option<Uuid>) -> Result<Vec<ConfigItemDto>, Supg006Error> {
        self.save_config(cmd, actor_id).await
    }
}
