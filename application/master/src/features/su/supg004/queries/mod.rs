//! SUPG004 Queries trait and Sea-ORM implementation

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::su::supg004::errors::Supg004Error;
use crate::features::su::supg004::models::{
    EmployeeOptionDto, EmployeeOptionsQuery, GroupOptionDto, ScopeOptionsDto, UserAccountDto, UserFilterQuery,
    UserPageDto,
};

pub mod db_supg004_queries;
mod employee_name_lookup;
mod employee_options;
mod get_user;
mod group_options;
mod list_users;
mod scope_names;
mod scope_options;
mod user_dto_lookup;

pub use db_supg004_queries::DbSupg004Queries;

/// What the identity provider needs to know about an employee
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmployeeNames {
    pub first_name: String,
    pub last_name: String,
}

#[async_trait]
pub trait Supg004Queries: Send + Sync {
    async fn list_users(&self, filter: UserFilterQuery) -> Result<UserPageDto, Supg004Error>;
    async fn get_user(&self, id: Uuid) -> Result<Option<UserAccountDto>, Supg004Error>;

    /// Active employees that have no account yet, matching the search (a few at a time)
    async fn employee_options(&self, query: EmployeeOptionsQuery) -> Result<Vec<EmployeeOptionDto>, Supg004Error>;

    async fn group_options(&self) -> Result<Vec<GroupOptionDto>, Supg004Error>;

    /// Organizations, companies, branches and departments for the assignment picker
    async fn scope_options(&self) -> Result<ScopeOptionsDto, Supg004Error>;

    /// English name (Thai when there is none) of an employee, for the identity's name traits
    async fn employee_names(&self, employee_id: Uuid) -> Result<Option<EmployeeNames>, Supg004Error>;
}
