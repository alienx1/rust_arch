//! SUPG004 Command Executor: validates account changes and coordinates the local `su_user` row
//! with the identity provider (Ory Kratos). Passwords are never handled here.
//!
//! Each action lives in its own file (`create_user`, `update_user`, `create_recovery_link`, `delete_user`); this
//! file only wires them to the `Supg004Commands` trait. Reads go straight to the queries.

use application_core::traits::IdentityProvisioningPort;
use application_core::traits::{ReadCaches, cache_ns, cache_ttl};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

use crate::features::su::account_security::{AccountSecurityCommands, PasswordMailResult};
use crate::features::su::supg004::commands::Supg004Commands;
use crate::features::su::supg004::errors::Supg004Error;
use crate::features::su::supg004::models::{
    CreateUserCommand, CreateUserResultDto, EmployeeOptionDto, EmployeeOptionsQuery,
    GroupOptionDto, ScopeOptionsDto, UpdateUserCommand, UserAccountDto, UserFilterQuery,
    UserPageDto,
};
use crate::features::su::supg004::queries::Supg004Queries;
use crate::features::su::supg005::commands::Supg005Commands;

pub struct Supg004CommandExecutor {
    pub(super) queries: Arc<dyn Supg004Queries>,
    pub(super) db: DatabaseConnection,
    /// `None` when no identity provider is configured: accounts cannot be created or recovered
    pub(super) identity: Option<Arc<dyn IdentityProvisioningPort>>,
    /// Switching an account on/off (identity provider first, then the local flag)
    pub(super) sessions: Arc<dyn Supg005Commands>,
    /// Valkey read-through for the rarely changing reads (off until `with_cache`)
    pub(super) caches: ReadCaches,
    /// Password mails and the forced change (`None`: no mail can be sent)
    pub(super) account_security: Option<Arc<dyn AccountSecurityCommands>>,
}

impl Supg004CommandExecutor {
    pub fn new(
        queries: Arc<dyn Supg004Queries>,
        db: DatabaseConnection,
        identity: Option<Arc<dyn IdentityProvisioningPort>>,
        sessions: Arc<dyn Supg005Commands>,
    ) -> Self {
        Self { queries, db, identity, sessions, caches: ReadCaches::disabled(), account_security: None }
    }

    /// Sends the password mails of new accounts and runs the forced change
    pub fn with_account_security(mut self, security: Arc<dyn AccountSecurityCommands>) -> Self {
        self.account_security = Some(security);
        self
    }

    pub(super) fn security(&self) -> Result<&Arc<dyn AccountSecurityCommands>, Supg004Error> {
        self.account_security
            .as_ref()
            .ok_or_else(|| Supg004Error::IdentityProvider("password mail is not configured".to_string()))
    }

    /// Reads that change rarely are served from Valkey; writes clear them (see the cache-invalidation middleware)
    pub fn with_cache(mut self, caches: ReadCaches) -> Self {
        self.caches = caches;
        self
    }
}

#[async_trait]
impl Supg004Commands for Supg004CommandExecutor {
    async fn list_users(&self, filter: UserFilterQuery) -> Result<UserPageDto, Supg004Error> {
        self.queries.list_users(filter).await
    }

    async fn get_user(&self, id: Uuid) -> Result<Option<UserAccountDto>, Supg004Error> {
        self.queries.get_user(id).await
    }

    async fn employee_options(&self, query: EmployeeOptionsQuery) -> Result<Vec<EmployeeOptionDto>, Supg004Error> {
        let cache = self.caches.ns(cache_ns::ORG);
        let key = cache.key("employee_options", &query);
        cache.through_key(key, cache_ttl::ORG, || self.queries.employee_options(query)).await
    }

    async fn group_options(&self) -> Result<Vec<GroupOptionDto>, Supg004Error> {
        let cache = self.caches.ns(cache_ns::ORG);
        let key = cache.key("group_options", &());
        cache.through_key(key, cache_ttl::ORG, || self.queries.group_options()).await
    }

    async fn scope_options(&self) -> Result<ScopeOptionsDto, Supg004Error> {
        let cache = self.caches.ns(cache_ns::ORG);
        let key = cache.key("scope_options", &());
        cache.through_key(key, cache_ttl::ORG, || self.queries.scope_options()).await
    }

    async fn create_user(
        &self,
        cmd: CreateUserCommand,
        actor_id: Option<Uuid>,
        can_assign_admin: bool,
    ) -> Result<CreateUserResultDto, Supg004Error> {
        self.create_user(cmd, actor_id, can_assign_admin).await
    }

    async fn update_user(
        &self,
        id: Uuid,
        cmd: UpdateUserCommand,
        actor_id: Option<Uuid>,
        can_assign_admin: bool,
    ) -> Result<UserAccountDto, Supg004Error> {
        self.update_user(id, cmd, actor_id, can_assign_admin).await
    }

    async fn send_password_mail(&self, id: Uuid) -> Result<PasswordMailResult, Supg004Error> {
        self.send_password_mail(id).await
    }

    async fn force_password_change(&self, id: Uuid) -> Result<PasswordMailResult, Supg004Error> {
        self.force_password_change(id).await
    }

    async fn delete_user(&self, id: Uuid, actor_id: Option<Uuid>) -> Result<(), Supg004Error> {
        self.delete_user(id, actor_id).await
    }
}
