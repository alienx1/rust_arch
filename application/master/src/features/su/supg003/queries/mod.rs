//! SUPG003 Queries trait and Sea-ORM implementation

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::su::supg003::errors::Supg003Error;
use crate::features::su::supg003::models::{GroupFilterQuery, PermissionGroupDto, PermissionMatrixDto};

pub mod db_supg003_queries;
mod get_group;
mod group_dto_lookup;
mod group_holders;
mod group_matrix;
mod list_groups;
mod program_tree_order;

pub use db_supg003_queries::{DbSupg003Queries, order_programs};
pub use group_holders::{group_user_ids, user_counts};

#[async_trait]
pub trait Supg003Queries: Send + Sync {
    async fn list_groups(&self, filter: GroupFilterQuery) -> Result<Vec<PermissionGroupDto>, Supg003Error>;
    async fn get_group(&self, id: Uuid) -> Result<Option<PermissionGroupDto>, Supg003Error>;

    /// The group's program × action matrix (every program, tree order, the group's active grants)
    async fn group_matrix(&self, id: Uuid) -> Result<Option<PermissionMatrixDto>, Supg003Error>;
}
