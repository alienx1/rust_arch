//! DBPG004 Queries trait and Sea-ORM implementation

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::db::dbpg004::errors::Dbpg004Error;
use crate::features::db::dbpg004::models::{
    Dbpg004SummaryDto, LookupCategoryDto, LookupDto, LookupFilterQuery, LookupPageDto,
};

pub mod db_dbpg004_queries;
mod get_lookup;
mod get_lookup_summary;
mod list_lookup_categories;
mod list_lookups;
mod lookup_dto_lookup;

pub use db_dbpg004_queries::DbDbpg004Queries;

#[async_trait]
pub trait Dbpg004Queries: Send + Sync {
    /// Categories with row counts, by name
    async fn list_categories(&self) -> Result<Vec<LookupCategoryDto>, Dbpg004Error>;

    /// One page of lookups matching the filter
    async fn list_lookups(&self, filter: LookupFilterQuery) -> Result<LookupPageDto, Dbpg004Error>;

    /// Retrieve a single lookup by ID
    async fn get_lookup_by_id(&self, id: Uuid) -> Result<Option<LookupDto>, Dbpg004Error>;

    /// KPI summary
    async fn get_summary(&self) -> Result<Dbpg004SummaryDto, Dbpg004Error>;
}
