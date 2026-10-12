//! Get one province / district / sub-district by id

use sea_orm::EntityTrait;
use uuid::Uuid;

use super::db_dbpg005_queries::DbDbpg005Queries;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{GeoNodeDto, db_district, db_province, db_sub_district};

impl DbDbpg005Queries {
    pub(super) async fn get_province(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error> {
        match db_province::Entity::find_by_id(id).one(&self.db).await? {
            Some(r) => Ok(self.province_nodes(vec![r]).await?.into_iter().next()),
            None => Ok(None),
        }
    }

    pub(super) async fn get_district(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error> {
        match db_district::Entity::find_by_id(id).one(&self.db).await? {
            Some(r) => Ok(self.district_nodes(vec![r]).await?.into_iter().next()),
            None => Ok(None),
        }
    }

    pub(super) async fn get_sub_district(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error> {
        match db_sub_district::Entity::find_by_id(id).one(&self.db).await? {
            Some(r) => Ok(self.sub_district_nodes(vec![r]).await?.into_iter().next()),
            None => Ok(None),
        }
    }
}
