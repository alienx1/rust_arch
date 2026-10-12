//! SUPG001 Commands trait (reads + writes consumed by routers) and executor

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{
    CreateModuleCommand, CreateSystemCommand, ModuleDto, ModuleFilterQuery, SystemDto,
    SystemFilterQuery, UpdateUnitCommand,
};

mod create_module;
mod create_system;
mod delete_module;
mod delete_system;
mod get_module;
mod get_system;
mod list_modules;
mod list_systems;
pub mod supg001_executor;
mod unit_label_writes;
mod unit_rules;
mod update_module;
mod update_system;

pub use supg001_executor::Supg001CommandExecutor;
pub use unit_rules::normalize_unit_code;

#[async_trait]
pub trait Supg001Commands: Send + Sync {
    async fn list_systems(&self, filter: SystemFilterQuery) -> Result<Vec<SystemDto>, Supg001Error>;
    async fn get_system(&self, id: Uuid) -> Result<Option<SystemDto>, Supg001Error>;
    async fn list_modules(&self, filter: ModuleFilterQuery) -> Result<Vec<ModuleDto>, Supg001Error>;
    async fn get_module(&self, id: Uuid) -> Result<Option<ModuleDto>, Supg001Error>;

    /// Create a system; the code must be unique
    async fn create_system(
        &self,
        cmd: CreateSystemCommand,
        actor_id: Option<Uuid>,
    ) -> Result<SystemDto, Supg001Error>;

    /// Update names / status with optimistic lock; protected systems cannot be switched off
    async fn update_system(
        &self,
        id: Uuid,
        cmd: UpdateUnitCommand,
        actor_id: Option<Uuid>,
    ) -> Result<SystemDto, Supg001Error>;

    /// Create a module; the code must be unique within the system
    async fn create_module(
        &self,
        cmd: CreateModuleCommand,
        actor_id: Option<Uuid>,
    ) -> Result<ModuleDto, Supg001Error>;

    /// Update names / status with optimistic lock; protected modules cannot be switched off
    async fn update_module(
        &self,
        id: Uuid,
        cmd: UpdateUnitCommand,
        actor_id: Option<Uuid>,
    ) -> Result<ModuleDto, Supg001Error>;

    /// Hard-delete a system and its labels; protected systems are refused and a system that is
    /// still referenced (modules, programs, grants) is `InUse`
    async fn delete_system(&self, id: Uuid) -> Result<(), Supg001Error>;

    /// Hard-delete a module and its labels; protected modules are refused and a module that is
    /// still referenced (programs, grants) is `InUse`
    async fn delete_module(&self, id: Uuid) -> Result<(), Supg001Error>;
}
