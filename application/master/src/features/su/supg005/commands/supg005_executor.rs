//! IAM Command Executor

use async_trait::async_trait;
use std::sync::Arc;

use crate::features::su::supg005::commands::{SessionRevocationPort, Supg005Commands};
use crate::features::su::supg005::errors::Supg005Error;
use crate::features::su::supg005::models::{PaginatedResult, UserPaginationQuery, UserSessionOverviewDto};
use crate::features::su::supg005::queries::Supg005Queries;

/// Executor coordinating IAM commands and queries
pub struct Supg005CommandExecutor {
    pub(super) queries: Arc<dyn Supg005Queries>,
    pub(super) revocation: Option<Arc<dyn SessionRevocationPort>>,
}

impl Supg005CommandExecutor {
    pub fn new(queries: Arc<dyn Supg005Queries>) -> Self {
        Self {
            queries,
            revocation: None,
        }
    }

    pub fn with_revocation(mut self, revocation: Option<Arc<dyn SessionRevocationPort>>) -> Self {
        self.revocation = revocation;
        self
    }
}

#[async_trait]
impl Supg005Commands for Supg005CommandExecutor {
    async fn list_users(&self) -> Result<Vec<UserSessionOverviewDto>, Supg005Error> {
        self.list_user_sessions().await
    }

    async fn list_users_paginated(
        &self,
        query: UserPaginationQuery,
    ) -> Result<PaginatedResult<UserSessionOverviewDto>, Supg005Error> {
        self.list_user_sessions_page(query).await
    }

    async fn revoke_user_sessions(&self, user_id: &str) -> Result<bool, Supg005Error> {
        self.revoke_all_sessions(user_id).await
    }

    async fn suspend_user(&self, user_id: &str) -> Result<bool, Supg005Error> {
        self.suspend_account(user_id).await
    }

    async fn reactivate_user(&self, user_id: &str) -> Result<bool, Supg005Error> {
        self.reactivate_account(user_id).await
    }

    async fn authorize_login(&self, identifier: &str) -> Result<bool, Supg005Error> {
        self.check_login_allowed(identifier).await
    }

    async fn revoke_device_session(&self, session_id: &str) -> Result<bool, Supg005Error> {
        self.revoke_session(session_id).await
    }
}
