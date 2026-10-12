//! SUPG001 get one system by id

use sea_orm::EntityTrait;
use uuid::Uuid;

use super::db_supg001_queries::DbSupg001Queries;
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{SystemDto, db_system};

impl DbSupg001Queries {
    pub(super) async fn get_system(&self, id: Uuid) -> Result<Option<SystemDto>, Supg001Error> {
        match db_system::Entity::find_by_id(id).one(&self.db).await? {
            Some(s) => Ok(self.system_dtos(vec![s], "").await?.into_iter().next()),
            None => Ok(None),
        }
    }
}
