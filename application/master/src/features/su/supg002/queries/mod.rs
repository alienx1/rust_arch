//! SUPG002 Queries trait and Sea-ORM implementation

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::{
    MenuNodeDto, ProgramDto, ProgramFilterQuery, ProgramOptionsDto, ProgramPageDto,
};

pub mod db_supg002_queries;
mod get_program;
mod label_lookup;
mod list_programs;
mod menu_tree;
mod menu_tree_builder;
mod program_dtos;
mod program_options;

pub use db_supg002_queries::DbSupg002Queries;
pub use menu_tree_builder::build_menu_tree;

#[async_trait]
pub trait Supg002Queries: Send + Sync {
    /// Menu tree for the account with this `su_user.username`: active programs (in active
    /// systems/modules) the account's permission group may READ, plus the folders above them.
    /// Unknown or inactive accounts get an empty tree.
    async fn menu_tree(&self, username: &str) -> Result<Vec<MenuNodeDto>, Supg002Error>;

    /// One page of programs matching the filter (management list; not permission-filtered)
    async fn list_programs(&self, filter: ProgramFilterQuery) -> Result<ProgramPageDto, Supg002Error>;

    /// Retrieve a single program by ID
    async fn get_program(&self, id: Uuid) -> Result<Option<ProgramDto>, Supg002Error>;

    /// Systems, modules and programs for the form selectors
    async fn program_options(&self) -> Result<ProgramOptionsDto, Supg002Error>;
}
