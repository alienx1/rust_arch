//! IAM Queries Trait and Sea-ORM Implementation

use async_trait::async_trait;

use crate::features::su::supg005::errors::Supg005Error;
use crate::features::su::supg005::models::{PaginatedResult, UserPaginationQuery, UserSessionOverviewDto};

pub mod db_supg005_queries;
mod find_user_login_identifier;
mod is_login_allowed;
mod list_users_paginated;
mod list_users_with_sessions;
mod set_user_active;
mod user_overview_names;
mod user_primary_scope;

pub use db_supg005_queries::DbSupg005Queries;

#[async_trait]
pub trait Supg005Queries: Send + Sync {
    /// Retrieve all enterprise users along with active device sessions
    async fn list_users_with_sessions(&self) -> Result<Vec<UserSessionOverviewDto>, Supg005Error>;

    /// Retrieve enterprise users paginated using SQL LIMIT and OFFSET
    async fn list_users_paginated(
        &self,
        query: UserPaginationQuery,
    ) -> Result<PaginatedResult<UserSessionOverviewDto>, Supg005Error>;

    /// Persist the account active flag. Returns `false` when the user does not exist.
    async fn set_user_active(&self, user_id: &str, active: bool) -> Result<bool, Supg005Error>;

    /// Whether the account behind a login identifier (email or username) may sign in: the user, its
    /// organization and its company must all be active.
    /// Only an existing `su_user` row with `active = false` is refused; unknown identifiers pass.
    async fn is_login_allowed(&self, identifier: &str) -> Result<bool, Supg005Error>;

    /// Login identifier (the username) used to match the local user to its identity-provider account.
    async fn find_user_login_identifier(&self, _user_id: &str) -> Result<Option<String>, Supg005Error> {
        Ok(None)
    }
}
