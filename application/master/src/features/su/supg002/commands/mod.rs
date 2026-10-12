//! SUPG002 Commands trait (menu tree + program management consumed by routers) and executor

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::{
    CreateProgramCommand, MenuNodeDto, ProgramDto, ProgramFilterQuery, ProgramOptionsDto,
    ProgramPageDto, UpdateProgramCommand,
};

mod check_parent;
mod create_program;
mod delete_program;
mod get_menu_tree;
mod get_program;
mod list_programs;
mod program_label_writes;
mod program_options;
mod program_rules;
pub mod supg002_executor;
mod update_program;

pub use program_rules::{normalize_program_code, normalize_program_path};
pub use supg002_executor::Supg002CommandExecutor;

#[async_trait]
pub trait Supg002Commands: Send + Sync {
    /// Menu tree for the account with this `su_user.username`
    async fn get_menu_tree(&self, username: &str) -> Result<Vec<MenuNodeDto>, Supg002Error>;

    /// One page of programs (management list)
    async fn list_programs(&self, filter: ProgramFilterQuery) -> Result<ProgramPageDto, Supg002Error>;

    async fn get_program(&self, id: Uuid) -> Result<Option<ProgramDto>, Supg002Error>;

    /// Systems, modules and programs for the form selectors
    async fn program_options(&self) -> Result<ProgramOptionsDto, Supg002Error>;

    /// Create a program; `(system, module, code)` must be unique
    async fn create_program(
        &self,
        cmd: CreateProgramCommand,
        actor_id: Option<Uuid>,
    ) -> Result<ProgramDto, Supg002Error>;

    /// Update place / route / icon / order / status / names with optimistic lock; protected
    /// programs cannot be switched off and a program cannot move below itself
    async fn update_program(
        &self,
        id: Uuid,
        cmd: UpdateProgramCommand,
        actor_id: Option<Uuid>,
    ) -> Result<ProgramDto, Supg002Error>;

    /// Hard-delete a program with its labels and permission grants; protected programs are
    /// refused and a program that still has child programs is `InUse`
    async fn delete_program(&self, id: Uuid) -> Result<(), Supg002Error>;
}
