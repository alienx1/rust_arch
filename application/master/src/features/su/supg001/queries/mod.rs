//! SUPG001 Queries trait and Sea-ORM implementation

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{
    ModuleDto, ModuleFilterQuery, SystemDto, SystemFilterQuery,
};

pub mod db_supg001_queries;
mod get_module;
mod get_system;
mod label_lookup;
mod list_modules;
mod list_systems;
mod module_dtos;
mod system_dtos;

pub use db_supg001_queries::DbSupg001Queries;

#[async_trait]
pub trait Supg001Queries: Send + Sync {
    async fn list_systems(&self, filter: SystemFilterQuery) -> Result<Vec<SystemDto>, Supg001Error>;
    async fn get_system(&self, id: Uuid) -> Result<Option<SystemDto>, Supg001Error>;
    async fn list_modules(&self, filter: ModuleFilterQuery) -> Result<Vec<ModuleDto>, Supg001Error>;
    async fn get_module(&self, id: Uuid) -> Result<Option<ModuleDto>, Supg001Error>;
}
