//! DBPG006 Queries trait and Sea-ORM implementation

use async_trait::async_trait;

use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::{
    LabelCategoryDto, LabelEntryDto, LabelFilterQuery, LabelPageDto, LanguageOptionDto,
};

pub mod db_dbpg006_queries;
mod get_label;
mod label_entry_assembly;
mod list_categories;
mod list_labels;
mod list_languages;

pub use db_dbpg006_queries::DbDbpg006Queries;

#[async_trait]
pub trait Dbpg006Queries: Send + Sync {
    /// Active languages with their Thai / English display names
    async fn list_languages(&self) -> Result<Vec<LanguageOptionDto>, Dbpg006Error>;

    /// Categories with their entry counts, by name
    async fn list_categories(&self) -> Result<Vec<LabelCategoryDto>, Dbpg006Error>;

    /// One page of entries matching the filter
    async fn list_labels(&self, filter: LabelFilterQuery) -> Result<LabelPageDto, Dbpg006Error>;

    /// One entry with all its translations
    async fn get_label(
        &self,
        category: &str,
        key: &str,
    ) -> Result<Option<LabelEntryDto>, Dbpg006Error>;
}
