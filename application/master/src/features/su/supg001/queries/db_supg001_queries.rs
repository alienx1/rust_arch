//! Sea-ORM query runner for systems and modules (small tables: lists are not paged)

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{ModuleDto, ModuleFilterQuery, SystemDto, SystemFilterQuery};

#[derive(Clone)]
pub struct DbSupg001Queries {
    pub(super) db: DatabaseConnection,
}

impl DbSupg001Queries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl super::Supg001Queries for DbSupg001Queries {
    async fn list_systems(&self, filter: SystemFilterQuery) -> Result<Vec<SystemDto>, Supg001Error> {
        self.list_systems(filter).await
    }

    async fn get_system(&self, id: Uuid) -> Result<Option<SystemDto>, Supg001Error> {
        self.get_system(id).await
    }

    async fn list_modules(&self, filter: ModuleFilterQuery) -> Result<Vec<ModuleDto>, Supg001Error> {
        self.list_modules(filter).await
    }

    async fn get_module(&self, id: Uuid) -> Result<Option<ModuleDto>, Supg001Error> {
        self.get_module(id).await
    }
}
