//! Sea-ORM implementation of the account security queries

use std::collections::HashMap;

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use super::{AccountRef, AccountSecurityQueries};
use crate::features::su::account_security::errors::AccountSecurityError;
use domain::entity::{su_configuration, su_user};

pub struct DbAccountSecurityQueries {
    db: DatabaseConnection,
}

impl DbAccountSecurityQueries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn account(u: su_user::Model) -> AccountRef {
    AccountRef { id: u.id, username: u.username, active: u.active, must_change_password: u.must_change_password }
}

#[async_trait]
impl AccountSecurityQueries for DbAccountSecurityQueries {
    async fn find_account(&self, id: Uuid) -> Result<Option<AccountRef>, AccountSecurityError> {
        Ok(su_user::Entity::find_by_id(id).one(&self.db).await?.map(account))
    }

    async fn find_account_by_identifier(&self, identifier: &str) -> Result<Option<AccountRef>, AccountSecurityError> {
        // Same convention as the login guard: the username, or the local part of an e-mail
        let username = identifier.trim().split('@').next().unwrap_or_default().to_lowercase();
        Ok(su_user::Entity::find()
            .filter(su_user::Column::Username.eq(username))
            .one(&self.db)
            .await?
            .map(account))
    }

    async fn set_must_change_password(&self, id: Uuid, value: bool) -> Result<bool, AccountSecurityError> {
        if su_user::Entity::find_by_id(id).one(&self.db).await?.is_none() {
            return Ok(false);
        }
        let mut row = su_user::ActiveModel { id: Set(id), ..Default::default() };
        row.must_change_password = Set(value);
        row.update(&self.db).await?;
        Ok(true)
    }

    async fn mail_template_configuration(&self) -> Result<HashMap<String, String>, AccountSecurityError> {
        Ok(su_configuration::Entity::find()
            .filter(su_configuration::Column::Category.eq("mail_template"))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|c| (c.key, c.value))
            .collect())
    }

    async fn security_configuration(&self) -> Result<HashMap<String, String>, AccountSecurityError> {
        Ok(su_configuration::Entity::find()
            .filter(su_configuration::Column::Category.eq("security"))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|c| (c.key, c.value))
            .collect())
    }
}
