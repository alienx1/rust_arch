//! Get one postal code by id

use sea_orm::EntityTrait;
use uuid::Uuid;

use super::db_dbpg005_queries::DbDbpg005Queries;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{PostalCodeDto, db_postal_code};

impl DbDbpg005Queries {
    pub(super) async fn get_postal_code(&self, id: Uuid) -> Result<Option<PostalCodeDto>, Dbpg005Error> {
        match db_postal_code::Entity::find_by_id(id).one(&self.db).await? {
            Some(r) => Ok(self.postal_dtos(vec![r]).await?.into_iter().next()),
            None => Ok(None),
        }
    }
}
