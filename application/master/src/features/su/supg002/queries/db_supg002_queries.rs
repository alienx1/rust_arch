//! Sea-ORM query runner for the menu tree

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::{
    MenuNodeDto, ProgramDto, ProgramFilterQuery, ProgramOptionsDto, ProgramPageDto,
};

#[derive(Clone)]
pub struct DbSupg002Queries {
    pub(super) db: DatabaseConnection,
}

impl DbSupg002Queries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl super::Supg002Queries for DbSupg002Queries {
    async fn menu_tree(&self, username: &str) -> Result<Vec<MenuNodeDto>, Supg002Error> {
        self.menu_tree(username).await
    }

    async fn list_programs(&self, filter: ProgramFilterQuery) -> Result<ProgramPageDto, Supg002Error> {
        self.list_programs(filter).await
    }

    async fn get_program(&self, id: Uuid) -> Result<Option<ProgramDto>, Supg002Error> {
        self.get_program(id).await
    }

    async fn program_options(&self) -> Result<ProgramOptionsDto, Supg002Error> {
        self.program_options().await
    }
}
