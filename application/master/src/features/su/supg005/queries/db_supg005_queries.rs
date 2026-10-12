//! Sea-ORM Queries for IAM User Directory and Session Tracking

use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::features::su::supg005::errors::Supg005Error;
use crate::features::su::supg005::models::{PaginatedResult, UserPaginationQuery, UserSessionOverviewDto};
use crate::features::su::supg005::queries::Supg005Queries;

/// Direct Sea-ORM query runner for IAM Users and Roles
pub struct DbSupg005Queries {
    pub(super) db: DatabaseConnection,
}

impl DbSupg005Queries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Supg005Queries for DbSupg005Queries {
    async fn list_users_with_sessions(&self) -> Result<Vec<UserSessionOverviewDto>, Supg005Error> {
        self.list_users_with_sessions().await
    }

    async fn set_user_active(&self, user_id: &str, active: bool) -> Result<bool, Supg005Error> {
        self.set_user_active(user_id, active).await
    }

    async fn is_login_allowed(&self, identifier: &str) -> Result<bool, Supg005Error> {
        self.is_login_allowed(identifier).await
    }

    async fn find_user_login_identifier(&self, user_id: &str) -> Result<Option<String>, Supg005Error> {
        self.find_user_login_identifier(user_id).await
    }

    async fn list_users_paginated(
        &self,
        query: UserPaginationQuery,
    ) -> Result<PaginatedResult<UserSessionOverviewDto>, Supg005Error> {
        self.list_users_paginated(query).await
    }
}
