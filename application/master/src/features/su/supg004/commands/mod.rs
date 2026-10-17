//! SUPG004 Commands trait (reads + writes consumed by routers) and executor

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::su::account_security::PasswordMailResult;
use crate::features::su::supg004::errors::Supg004Error;
use crate::features::su::supg004::models::{
    CreateUserCommand, CreateUserResultDto, EmployeeOptionDto, EmployeeOptionsQuery,
    GroupOptionDto, ScopeOptionsDto, UpdateUserCommand, UserAccountDto, UserFilterQuery,
    UserPageDto,
};

mod assignment_rules;
mod password_mail_actions;
mod create_user;
mod delete_user;
mod group_lookup;
mod identity_rules;
pub mod supg004_executor;
mod update_user;

pub use assignment_rules::{Assignment, normalize_assignments, scope_level_of};
pub use identity_rules::{is_valid_email, normalize_username};
pub use supg004_executor::Supg004CommandExecutor;

#[async_trait]
pub trait Supg004Commands: Send + Sync {
    async fn list_users(&self, filter: UserFilterQuery) -> Result<UserPageDto, Supg004Error>;
    async fn get_user(&self, id: Uuid) -> Result<Option<UserAccountDto>, Supg004Error>;
    async fn employee_options(&self, query: EmployeeOptionsQuery) -> Result<Vec<EmployeeOptionDto>, Supg004Error>;
    async fn group_options(&self) -> Result<Vec<GroupOptionDto>, Supg004Error>;
    async fn scope_options(&self) -> Result<ScopeOptionsDto, Supg004Error>;

    /// Create the identity at the identity provider (no password), then the local account, then
    /// try to issue a recovery link. `can_assign_admin` is `true` when the caller holds APPROVE
    /// on SUPG004, which placing an account in the ADMIN group requires.
    async fn create_user(
        &self,
        cmd: CreateUserCommand,
        actor_id: Option<Uuid>,
        can_assign_admin: bool,
    ) -> Result<CreateUserResultDto, Supg004Error>;

    /// Replace an account's assignments and/or switch it on/off (optimistic lock). Switching off
    /// also blocks login and revokes the sessions at the identity provider. Callers cannot change
    /// their own assignments or status.
    async fn update_user(
        &self,
        id: Uuid,
        cmd: UpdateUserCommand,
        actor_id: Option<Uuid>,
        can_assign_admin: bool,
    ) -> Result<UserAccountDto, Supg004Error>;

    /// E-mail the account holder a one-time link to choose a new password (the old one keeps
    /// working until then). When the mail cannot be sent the result carries the link instead.
    async fn send_password_mail(&self, id: Uuid) -> Result<PasswordMailResult, Supg004Error>;

    /// Require a new password: the account cannot sign in until it is set, its sessions end and
    /// the link is e-mailed
    async fn force_password_change(&self, id: Uuid) -> Result<PasswordMailResult, Supg004Error>;

    /// Delete an account (not the caller's own) and then its sign-in identity at the identity
    /// provider. A record that is still referenced is refused as `InUse`.
    async fn delete_user(&self, id: Uuid, actor_id: Option<Uuid>) -> Result<(), Supg004Error>;
}
