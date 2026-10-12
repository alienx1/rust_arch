//! IAM Commands Trait and Executor Module

use async_trait::async_trait;

use crate::features::su::supg005::errors::Supg005Error;
use crate::features::su::supg005::models::{PaginatedResult, UserPaginationQuery, UserSessionOverviewDto};

mod account_activation_rules;
mod authorize_login;
mod list_user_sessions;
mod list_user_sessions_paginated;
mod live_device_mapping;
mod reactivate_user;
mod revoke_device_session;
mod revoke_user_sessions;
mod suspend_user;
pub mod supg005_executor;

pub use supg005_executor::Supg005CommandExecutor;

#[async_trait]
pub trait Supg005Commands: Send + Sync {
    /// Retrieve all enterprise users and active sessions
    async fn list_users(&self) -> Result<Vec<UserSessionOverviewDto>, Supg005Error>;

    /// Retrieve enterprise users paginated using SQL LIMIT and OFFSET
    async fn list_users_paginated(
        &self,
        query: UserPaginationQuery,
    ) -> Result<PaginatedResult<UserSessionOverviewDto>, Supg005Error>;

    /// Revoke all active sessions for a specific user
    async fn revoke_user_sessions(&self, user_id: &str) -> Result<bool, Supg005Error>;

    /// Suspend the account: block login at the identity provider, revoke all its sessions
    /// and mark the user inactive.
    async fn suspend_user(&self, user_id: &str) -> Result<bool, Supg005Error>;

    /// Lift a suspension so the user can log in again.
    async fn reactivate_user(&self, user_id: &str) -> Result<bool, Supg005Error>;

    /// Login gate called by the identity provider after credentials are verified.
    /// Returns `false` when the account is inactive and must not sign in.
    async fn authorize_login(&self, identifier: &str) -> Result<bool, Supg005Error>;

    /// Revoke a specific device session
    async fn revoke_device_session(&self, session_id: &str) -> Result<bool, Supg005Error>;
}

pub use application_core::traits::SessionRevocationPort;
