//! SUPG002 Command Executor: the menu tree (reads) and program management (validated writes, a
//! program and its labels in one transaction).

use application_core::traits::{ReadCaches, cache_ns, cache_ttl};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

use crate::features::su::supg002::commands::Supg002Commands;
use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::{
    CreateProgramCommand, MenuNodeDto, ProgramDto, ProgramFilterQuery, ProgramOptionsDto,
    ProgramPageDto, UpdateProgramCommand,
};
use crate::features::su::supg002::queries::Supg002Queries;

pub struct Supg002CommandExecutor {
    pub(super) queries: Arc<dyn Supg002Queries>,
    pub(super) db: DatabaseConnection,
    /// Valkey read-through for the rarely changing reads (off until `with_cache`)
    pub(super) caches: ReadCaches,
}

impl Supg002CommandExecutor {
    pub fn new(queries: Arc<dyn Supg002Queries>, db: DatabaseConnection) -> Self {
        Self { queries, db, caches: ReadCaches::disabled() }
    }

    /// Reads that change rarely are served from Valkey; writes clear them (see the cache-invalidation middleware)
    pub fn with_cache(mut self, caches: ReadCaches) -> Self {
        self.caches = caches;
        self
    }
}

#[async_trait]
impl Supg002Commands for Supg002CommandExecutor {
    async fn get_menu_tree(&self, username: &str) -> Result<Vec<MenuNodeDto>, Supg002Error> {
        let cache = self.caches.ns(cache_ns::MENU).language_free();
        let key = cache.key("get_menu_tree", &username);
        cache.through_key(key, cache_ttl::MENU, || self.get_menu_tree(username)).await
    }

    async fn list_programs(&self, filter: ProgramFilterQuery) -> Result<ProgramPageDto, Supg002Error> {
        self.list_programs(filter).await
    }

    async fn get_program(&self, id: Uuid) -> Result<Option<ProgramDto>, Supg002Error> {
        self.get_program(id).await
    }

    async fn program_options(&self) -> Result<ProgramOptionsDto, Supg002Error> {
        let cache = self.caches.ns(cache_ns::PROGRAMS);
        let key = cache.key("program_options", &());
        cache.through_key(key, cache_ttl::PROGRAMS, || self.program_options()).await
    }

    async fn create_program(
        &self,
        cmd: CreateProgramCommand,
        actor_id: Option<Uuid>,
    ) -> Result<ProgramDto, Supg002Error> {
        self.create_program(cmd, actor_id).await
    }

    async fn update_program(
        &self,
        id: Uuid,
        cmd: UpdateProgramCommand,
        actor_id: Option<Uuid>,
    ) -> Result<ProgramDto, Supg002Error> {
        self.update_program(id, cmd, actor_id).await
    }

    async fn delete_program(&self, id: Uuid) -> Result<(), Supg002Error> {
        self.delete_program(id).await
    }
}
