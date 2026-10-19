//! SUPG006 Commands trait (what the router consumes) and executor

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::su::supg006::errors::Supg006Error;
use crate::features::su::supg006::models::{ConfigItemDto, SaveConfigCommand};

mod list_config;
mod save_config;
pub mod supg006_executor;

pub use save_config::validate_change;
pub use supg006_executor::Supg006CommandExecutor;

#[async_trait]
pub trait Supg006Commands: Send + Sync {
    /// Every setting of this program with its stored value (or default) and row version
    async fn list_config(&self) -> Result<Vec<ConfigItemDto>, Supg006Error>;

    /// Saves the changed settings, all or none (optimistic lock per setting), and returns the
    /// settings as stored. A value is validated per key: numbers in range, templates free of
    /// unknown placeholders and with `{{link}}` in the bodies.
    async fn save_config(&self, cmd: SaveConfigCommand, actor_id: Option<Uuid>) -> Result<Vec<ConfigItemDto>, Supg006Error>;
}
