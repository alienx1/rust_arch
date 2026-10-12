//! Sea-ORM query runner for user accounts. Filtering, searching and paging run in SQL; names are
//! loaded only for the page being returned.
//!
//! Each query lives in its own file; this file only wires them to the `Supg004Queries` trait.

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use super::EmployeeNames;
use crate::features::su::supg004::errors::Supg004Error;
use crate::features::su::supg004::models::{
    EmployeeOptionDto, EmployeeOptionsQuery, GroupOptionDto, ScopeOptionsDto, UserAccountDto, UserFilterQuery,
    UserPageDto,
};

#[derive(Clone)]
pub struct DbSupg004Queries {
    pub(super) db: DatabaseConnection,
}

impl DbSupg004Queries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl super::Supg004Queries for DbSupg004Queries {
    async fn list_users(&self, filter: UserFilterQuery) -> Result<UserPageDto, Supg004Error> {
        self.list_users(filter).await
    }

    async fn get_user(&self, id: Uuid) -> Result<Option<UserAccountDto>, Supg004Error> {
        self.get_user(id).await
    }

    async fn employee_options(&self, query: EmployeeOptionsQuery) -> Result<Vec<EmployeeOptionDto>, Supg004Error> {
        self.employee_options(query).await
    }

    async fn group_options(&self) -> Result<Vec<GroupOptionDto>, Supg004Error> {
        self.group_options().await
    }

    async fn scope_options(&self) -> Result<ScopeOptionsDto, Supg004Error> {
        self.scope_options().await
    }

    async fn employee_names(&self, employee_id: Uuid) -> Result<Option<EmployeeNames>, Supg004Error> {
        self.employee_names(employee_id).await
    }
}
