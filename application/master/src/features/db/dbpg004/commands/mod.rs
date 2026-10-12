//! DBPG004 Commands trait (reads + writes consumed by routers) and executor

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::db::dbpg004::errors::Dbpg004Error;
use crate::features::db::dbpg004::models::{
    CreateLookupCommand, Dbpg004SummaryDto, LookupCategoryDto, LookupDto, LookupFilterQuery,
    LookupPageDto, UpdateLookupCommand,
};

pub mod dbpg004_executor;
mod create_lookup;
mod delete_lookup;
mod get_lookup;
mod get_lookup_summary;
mod list_lookup_categories;
mod list_lookups;
mod lookup_label_upsert;
mod lookup_rules;
mod update_lookup;

pub use dbpg004_executor::Dbpg004CommandExecutor;
pub use lookup_rules::{normalize_category, normalize_code};

#[async_trait]
pub trait Dbpg004Commands: Send + Sync {
    async fn list_categories(&self) -> Result<Vec<LookupCategoryDto>, Dbpg004Error>;
    async fn list_lookups(&self, filter: LookupFilterQuery) -> Result<LookupPageDto, Dbpg004Error>;
    async fn get_lookup_by_id(&self, id: Uuid) -> Result<Option<LookupDto>, Dbpg004Error>;
    async fn get_summary(&self) -> Result<Dbpg004SummaryDto, Dbpg004Error>;

    /// Create a lookup value (and its category when new); `(category, code)` must be unique
    async fn create_lookup(
        &self,
        cmd: CreateLookupCommand,
        actor_id: Option<Uuid>,
    ) -> Result<LookupDto, Dbpg004Error>;

    /// Update labels / active flag with optimistic lock verification
    async fn update_lookup(
        &self,
        id: Uuid,
        cmd: UpdateLookupCommand,
        actor_id: Option<Uuid>,
    ) -> Result<LookupDto, Dbpg004Error>;

    /// Delete a lookup value with its labels; protected values are refused, referenced ones
    /// fail with `InUse`
    async fn delete_lookup(&self, id: Uuid) -> Result<(), Dbpg004Error>;
}
