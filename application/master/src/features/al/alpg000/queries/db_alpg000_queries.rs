//! ALPG000 Sea-ORM query runner (catalogue programs for the filter)

use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::features::al::alpg000::errors::Alpg000Error;
use crate::features::al::alpg000::models::ProgramOptionDto;

#[derive(Clone)]
pub struct DbAlpg000Queries {
    pub(super) db: DatabaseConnection,
}

impl DbAlpg000Queries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl super::Alpg000Queries for DbAlpg000Queries {
    async fn list_program_options(&self) -> Result<Vec<ProgramOptionDto>, Alpg000Error> {
        self.program_options().await
    }
}
