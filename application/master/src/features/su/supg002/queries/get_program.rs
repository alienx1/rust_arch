//! SUPG002 get one program by id

use sea_orm::EntityTrait;
use uuid::Uuid;

use super::db_supg002_queries::DbSupg002Queries;
use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::{ProgramDto, db_program};

impl DbSupg002Queries {
    pub(super) async fn get_program(&self, id: Uuid) -> Result<Option<ProgramDto>, Supg002Error> {
        match db_program::Entity::find_by_id(id).one(&self.db).await? {
            Some(p) => Ok(self.program_dtos(vec![p]).await?.into_iter().next()),
            None => Ok(None),
        }
    }
}
