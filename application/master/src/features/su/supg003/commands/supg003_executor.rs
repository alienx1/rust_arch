//! SUPG003 Command Executor: validates and persists permission groups and matrices.
//!
//! Each action lives in its own file (`create_group`, `update_group`, `save_matrix`, `delete_group`); this file
//! only wires them to the `Supg003Commands` trait. Reads go straight to the queries.

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

use crate::features::su::supg003::commands::Supg003Commands;
use crate::features::su::supg003::errors::Supg003Error;
use crate::features::su::supg003::models::{
    CreatePermissionGroupCommand, GroupFilterQuery, PermissionGroupDto, PermissionMatrixDto,
    SaveMatrixCommand, UpdatePermissionGroupCommand,
};
use crate::features::su::supg003::queries::Supg003Queries;

pub struct Supg003CommandExecutor {
    pub(super) queries: Arc<dyn Supg003Queries>,
    pub(super) db: DatabaseConnection,
}

impl Supg003CommandExecutor {
    pub fn new(queries: Arc<dyn Supg003Queries>, db: DatabaseConnection) -> Self {
        Self { queries, db }
    }
}

#[async_trait]
impl Supg003Commands for Supg003CommandExecutor {
    async fn list_groups(&self, filter: GroupFilterQuery) -> Result<Vec<PermissionGroupDto>, Supg003Error> {
        self.queries.list_groups(filter).await
    }

    async fn get_group(&self, id: Uuid) -> Result<Option<PermissionGroupDto>, Supg003Error> {
        self.queries.get_group(id).await
    }

    async fn group_matrix(&self, id: Uuid) -> Result<Option<PermissionMatrixDto>, Supg003Error> {
        self.queries.group_matrix(id).await
    }

    async fn create_group(
        &self,
        cmd: CreatePermissionGroupCommand,
        actor_id: Option<Uuid>,
    ) -> Result<PermissionGroupDto, Supg003Error> {
        self.create_group(cmd, actor_id).await
    }

    async fn update_group(
        &self,
        id: Uuid,
        cmd: UpdatePermissionGroupCommand,
        actor_id: Option<Uuid>,
    ) -> Result<PermissionGroupDto, Supg003Error> {
        self.update_group(id, cmd, actor_id).await
    }

    async fn save_matrix(
        &self,
        id: Uuid,
        cmd: SaveMatrixCommand,
        actor_id: Option<Uuid>,
    ) -> Result<PermissionMatrixDto, Supg003Error> {
        self.save_matrix(id, cmd, actor_id).await
    }

    async fn delete_group(&self, id: Uuid) -> Result<(), Supg003Error> {
        self.delete_group(id).await
    }
}
