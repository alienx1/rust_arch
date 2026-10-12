//! Get one province / district / sub-district (delegates to `Dbpg005Queries`)

use uuid::Uuid;

use super::dbpg005_executor::Dbpg005CommandExecutor;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::GeoNodeDto;

impl Dbpg005CommandExecutor {
    pub(super) async fn get_province(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error> {
        self.queries.get_province(id).await
    }

    pub(super) async fn get_district(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error> {
        self.queries.get_district(id).await
    }

    pub(super) async fn get_sub_district(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error> {
        self.queries.get_sub_district(id).await
    }
}
