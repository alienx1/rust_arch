//! Sea-ORM query runner for geography. Filtering, searching and paging run in SQL; names and
//! parents are loaded only for the page being returned. Each query lives in its own file.

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{
    Dbpg005SummaryDto, GeoFilterQuery, GeoNodeDto, GeoPageDto, PostalCodeDto, PostalFilterQuery,
    PostalPageDto,
};

#[derive(Clone)]
pub struct DbDbpg005Queries {
    pub(super) db: DatabaseConnection,
}

impl DbDbpg005Queries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl super::Dbpg005Queries for DbDbpg005Queries {
    async fn list_provinces(&self, filter: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error> {
        self.list_provinces(filter).await
    }

    async fn list_districts(&self, filter: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error> {
        self.list_districts(filter).await
    }

    async fn list_sub_districts(&self, filter: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error> {
        self.list_sub_districts(filter).await
    }

    async fn list_postal_codes(&self, filter: PostalFilterQuery) -> Result<PostalPageDto, Dbpg005Error> {
        self.list_postal_codes(filter).await
    }

    async fn get_province(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error> {
        self.get_province(id).await
    }

    async fn get_district(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error> {
        self.get_district(id).await
    }

    async fn get_sub_district(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error> {
        self.get_sub_district(id).await
    }

    async fn get_postal_code(&self, id: Uuid) -> Result<Option<PostalCodeDto>, Dbpg005Error> {
        self.get_postal_code(id).await
    }

    async fn get_summary(&self) -> Result<Dbpg005SummaryDto, Dbpg005Error> {
        self.get_summary().await
    }
}
