//! DBPG004 query: retrieve a single lookup by ID

use sea_orm::EntityTrait;
use uuid::Uuid;

use crate::features::db::dbpg004::errors::Dbpg004Error;
use crate::features::db::dbpg004::models::{LookupDto, db_master_lookup};

use super::db_dbpg004_queries::DbDbpg004Queries;

impl DbDbpg004Queries {
    pub(super) async fn get_lookup_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<LookupDto>, Dbpg004Error> {
        match db_master_lookup::Entity::find_by_id(id)
            .one(&self.db)
            .await?
        {
            Some(row) => Ok(self.lookup_dtos(vec![row]).await?.into_iter().next()),
            None => Ok(None),
        }
    }
}
