//! SUPG001 get one module by id

use sea_orm::EntityTrait;
use uuid::Uuid;

use super::db_supg001_queries::DbSupg001Queries;
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{ModuleDto, db_module};

impl DbSupg001Queries {
    pub(super) async fn get_module(&self, id: Uuid) -> Result<Option<ModuleDto>, Supg001Error> {
        match db_module::Entity::find_by_id(id).one(&self.db).await? {
            Some(m) => Ok(self.module_dtos(vec![m], "").await?.into_iter().next()),
            None => Ok(None),
        }
    }
}
