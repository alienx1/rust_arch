//! SUPG001 Command Executor: validates and persists system/module writes (unit + labels in one
//! transaction). Reads delegate to `Supg001Queries`.

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

use crate::features::su::supg001::commands::Supg001Commands;
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{
    CreateModuleCommand, CreateSystemCommand, ModuleDto, ModuleFilterQuery, SystemDto,
    SystemFilterQuery, UpdateUnitCommand,
};
use crate::features::su::supg001::queries::Supg001Queries;

pub struct Supg001CommandExecutor {
    pub(super) queries: Arc<dyn Supg001Queries>,
    pub(super) db: DatabaseConnection,
}

impl Supg001CommandExecutor {
    pub fn new(queries: Arc<dyn Supg001Queries>, db: DatabaseConnection) -> Self {
        Self { queries, db }
    }
}

#[async_trait]
impl Supg001Commands for Supg001CommandExecutor {
    async fn list_systems(&self, filter: SystemFilterQuery) -> Result<Vec<SystemDto>, Supg001Error> {
        self.list_systems(filter).await
    }

    async fn get_system(&self, id: Uuid) -> Result<Option<SystemDto>, Supg001Error> {
        self.get_system(id).await
    }

    async fn list_modules(&self, filter: ModuleFilterQuery) -> Result<Vec<ModuleDto>, Supg001Error> {
        self.list_modules(filter).await
    }

    async fn get_module(&self, id: Uuid) -> Result<Option<ModuleDto>, Supg001Error> {
        self.get_module(id).await
    }

    async fn create_system(
        &self,
        cmd: CreateSystemCommand,
        actor_id: Option<Uuid>,
    ) -> Result<SystemDto, Supg001Error> {
        self.create_system(cmd, actor_id).await
    }

    async fn update_system(
        &self,
        id: Uuid,
        cmd: UpdateUnitCommand,
        actor_id: Option<Uuid>,
    ) -> Result<SystemDto, Supg001Error> {
        self.update_system(id, cmd, actor_id).await
    }

    async fn create_module(
        &self,
        cmd: CreateModuleCommand,
        actor_id: Option<Uuid>,
    ) -> Result<ModuleDto, Supg001Error> {
        self.create_module(cmd, actor_id).await
    }

    async fn update_module(
        &self,
        id: Uuid,
        cmd: UpdateUnitCommand,
        actor_id: Option<Uuid>,
    ) -> Result<ModuleDto, Supg001Error> {
        self.update_module(id, cmd, actor_id).await
    }

    async fn delete_system(&self, id: Uuid) -> Result<(), Supg001Error> {
        self.delete_system(id).await
    }

    async fn delete_module(&self, id: Uuid) -> Result<(), Supg001Error> {
        self.delete_module(id).await
    }
}
