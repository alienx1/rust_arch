//! Sea-ORM query runner for label entries. Filtering, searching and paging run in SQL over the
//! distinct `(category, key)` pairs; translations are loaded only for the page being returned.

use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::{
    LabelCategoryDto, LabelEntryDto, LabelFilterQuery, LabelPageDto, LanguageOptionDto,
};

#[derive(Clone)]
pub struct DbDbpg006Queries {
    pub(super) db: DatabaseConnection,
}

impl DbDbpg006Queries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl super::Dbpg006Queries for DbDbpg006Queries {
    async fn list_languages(&self) -> Result<Vec<LanguageOptionDto>, Dbpg006Error> {
        self.list_languages().await
    }

    async fn list_categories(&self) -> Result<Vec<LabelCategoryDto>, Dbpg006Error> {
        self.list_categories().await
    }

    async fn list_labels(&self, filter: LabelFilterQuery) -> Result<LabelPageDto, Dbpg006Error> {
        self.list_labels(filter).await
    }

    async fn get_label(
        &self,
        category: &str,
        key: &str,
    ) -> Result<Option<LabelEntryDto>, Dbpg006Error> {
        self.get_label(category, key).await
    }
}
