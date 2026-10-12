//! ALPG000 Queries trait: the catalogue programs offered by the program filter

use async_trait::async_trait;

use crate::features::al::alpg000::errors::Alpg000Error;
use crate::features::al::alpg000::models::ProgramOptionDto;

pub mod db_alpg000_queries;
mod list_program_options;

pub use db_alpg000_queries::DbAlpg000Queries;

#[async_trait]
pub trait Alpg000Queries: Send + Sync {
    /// Programs that have a page, ordered by code
    async fn list_program_options(&self) -> Result<Vec<ProgramOptionDto>, Alpg000Error>;
}
