//! DBPG006 Commands trait (reads + writes consumed by routers) and executor

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::{
    CreateLabelCommand, LabelCategoryDto, LabelEntryDto, LabelFilterQuery, LabelPageDto,
    LanguageOptionDto, UpdateLabelCommand,
};

mod create_label;
mod delete_label;
pub mod dbpg006_executor;
mod get_label;
mod label_rules;
mod label_value_writer;
mod list_categories;
mod list_labels;
mod list_languages;
mod update_label;

pub use dbpg006_executor::Dbpg006CommandExecutor;
pub use label_rules::{normalize_label_category, normalize_label_key};

#[async_trait]
pub trait Dbpg006Commands: Send + Sync {
    /// Active languages (one input per language in the form)
    async fn list_languages(&self) -> Result<Vec<LanguageOptionDto>, Dbpg006Error>;

    /// Categories with their entry counts
    async fn list_categories(&self) -> Result<Vec<LabelCategoryDto>, Dbpg006Error>;

    /// One page of entries (`category`, `search`, `page`, `limit`)
    async fn list_labels(&self, filter: LabelFilterQuery) -> Result<LabelPageDto, Dbpg006Error>;

    /// One entry with all its translations
    async fn get_label(
        &self,
        category: &str,
        key: &str,
    ) -> Result<Option<LabelEntryDto>, Dbpg006Error>;

    /// Create an entry; `Duplicate` when `(category, key)` already has a row
    async fn create_label(
        &self,
        cmd: CreateLabelCommand,
        actor_id: Option<Uuid>,
    ) -> Result<LabelEntryDto, Dbpg006Error>;

    /// Replace the translations of an entry: a blank value removes that language's row
    async fn update_label(
        &self,
        cmd: UpdateLabelCommand,
        actor_id: Option<Uuid>,
    ) -> Result<LabelEntryDto, Dbpg006Error>;

    /// Delete every row of an entry; `NotFound` when it has none
    async fn delete_label(&self, category: &str, key: &str) -> Result<(), Dbpg006Error>;
}
