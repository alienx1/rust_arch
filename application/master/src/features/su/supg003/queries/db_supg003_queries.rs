//! Sea-ORM query runner for permission groups and matrices (small tables: lists are not paged).
//!
//! Each query lives in its own file; this file only wires them to the `Supg003Queries` trait.

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::features::su::supg003::errors::Supg003Error;
use crate::features::su::supg003::models::{GroupFilterQuery, PermissionGroupDto, PermissionMatrixDto};

pub use super::program_tree_order::order_programs;

#[derive(Clone)]
pub struct DbSupg003Queries {
    pub(super) db: DatabaseConnection,
}

impl DbSupg003Queries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl super::Supg003Queries for DbSupg003Queries {
    async fn list_groups(&self, filter: GroupFilterQuery) -> Result<Vec<PermissionGroupDto>, Supg003Error> {
        self.list_groups(filter).await
    }

    async fn get_group(&self, id: Uuid) -> Result<Option<PermissionGroupDto>, Supg003Error> {
        self.get_group(id).await
    }

    async fn group_matrix(&self, id: Uuid) -> Result<Option<PermissionMatrixDto>, Supg003Error> {
        self.group_matrix(id).await
    }
}
