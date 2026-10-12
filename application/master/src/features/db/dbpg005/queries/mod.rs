//! DBPG005 Queries trait and Sea-ORM implementation

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{
    Dbpg005SummaryDto, GeoFilterQuery, GeoNodeDto, GeoPageDto, PostalCodeDto, PostalFilterQuery,
    PostalPageDto,
};

pub mod db_dbpg005_queries;

mod geo_node_mapping;
mod get_geo_node;
mod get_postal_code;
mod get_summary;
mod label_lookup;
mod list_districts;
mod list_postal_codes;
mod list_provinces;
mod list_sub_districts;
mod search_paging;

pub use db_dbpg005_queries::DbDbpg005Queries;

#[async_trait]
pub trait Dbpg005Queries: Send + Sync {
    async fn list_provinces(&self, filter: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error>;
    async fn list_districts(&self, filter: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error>;
    async fn list_sub_districts(&self, filter: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error>;
    async fn list_postal_codes(
        &self,
        filter: PostalFilterQuery,
    ) -> Result<PostalPageDto, Dbpg005Error>;

    async fn get_province(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error>;
    async fn get_district(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error>;
    async fn get_sub_district(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error>;
    async fn get_postal_code(&self, id: Uuid) -> Result<Option<PostalCodeDto>, Dbpg005Error>;

    async fn get_summary(&self) -> Result<Dbpg005SummaryDto, Dbpg005Error>;
}
