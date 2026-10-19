//! Sea-ORM implementation of the SUPG006 queries

use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use super::Supg006Queries;
use crate::features::su::supg006::errors::Supg006Error;
use crate::features::su::supg006::models::CONFIG_SETTINGS;
use domain::entity::su_configuration;

pub struct DbSupg006Queries {
    db: DatabaseConnection,
}

impl DbSupg006Queries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Supg006Queries for DbSupg006Queries {
    async fn list_rows(&self) -> Result<Vec<su_configuration::Model>, Supg006Error> {
        let keys: Vec<&str> = CONFIG_SETTINGS.iter().map(|s| s.key).collect();
        Ok(su_configuration::Entity::find().filter(su_configuration::Column::Key.is_in(keys)).all(&self.db).await?)
    }
}
