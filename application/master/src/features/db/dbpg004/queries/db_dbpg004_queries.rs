//! Sea-ORM query runner for lookups. Filtering, searching and paging run in SQL; labels are
//! loaded only for the page being returned.

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::features::db::dbpg004::errors::Dbpg004Error;
use crate::features::db::dbpg004::models::{
    Dbpg004SummaryDto, LookupCategoryDto, LookupDto, LookupFilterQuery, LookupPageDto,
};

#[derive(Clone)]
pub struct DbDbpg004Queries {
    pub(super) db: DatabaseConnection,
}

impl DbDbpg004Queries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl super::Dbpg004Queries for DbDbpg004Queries {
    async fn list_categories(&self) -> Result<Vec<LookupCategoryDto>, Dbpg004Error> {
        self.list_categories().await
    }

    async fn list_lookups(&self, filter: LookupFilterQuery) -> Result<LookupPageDto, Dbpg004Error> {
        self.list_lookups(filter).await
    }

    async fn get_lookup_by_id(&self, id: Uuid) -> Result<Option<LookupDto>, Dbpg004Error> {
        self.get_lookup_by_id(id).await
    }

    async fn get_summary(&self) -> Result<Dbpg004SummaryDto, Dbpg004Error> {
        self.get_summary().await
    }
}
