//! List provinces / districts / sub-districts (delegates to `Dbpg005Queries`)

use super::dbpg005_executor::Dbpg005CommandExecutor;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{GeoFilterQuery, GeoPageDto};

impl Dbpg005CommandExecutor {
    pub(super) async fn list_provinces(&self, filter: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error> {
        self.queries.list_provinces(filter).await
    }

    pub(super) async fn list_districts(&self, filter: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error> {
        self.queries.list_districts(filter).await
    }

    pub(super) async fn list_sub_districts(
        &self,
        filter: GeoFilterQuery,
    ) -> Result<GeoPageDto, Dbpg005Error> {
        self.queries.list_sub_districts(filter).await
    }
}
