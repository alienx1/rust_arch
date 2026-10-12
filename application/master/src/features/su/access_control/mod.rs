//! Access control for the security-administration programs (SUPG001–SUPG004).
//!
//! A request is allowed when the signed-in account (`su_user.username`) is active, its permission
//! group is active and holds an active permission for the program and action (`permission_type`
//! lookup `READ` / `WRITE` / `APPROVE`). The result is the acting user, whose id is recorded as
//! `change_by` on every write.

mod access_snapshot;
mod scope_target;

use async_trait::async_trait;
use std::collections::{BTreeMap, BTreeSet};
use application_core::traits::{ReadCaches, cache_ns, cache_ttl};
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};
use thiserror::Error;
use uuid::Uuid;

use domain::entity::sea_orm_active_enums::ScopeLevel;
use domain::entity::{db_branch, db_company, db_department, db_organization};

pub use access_snapshot::{AccessSnapshot, Grant};
pub use scope_target::{ScopeTarget, grant_applies};

/// What the caller wants to do with a program
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PermissionAction {
    Read,
    Write,
    Approve,
}

impl PermissionAction {
    /// `db_master_lookup.code` of the action (category `permission_type`)
    pub fn code(self) -> &'static str {
        match self {
            PermissionAction::Read => "READ",
            PermissionAction::Write => "WRITE",
            PermissionAction::Approve => "APPROVE",
        }
    }
}

/// The authorized caller
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Actor {
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Debug, Error)]
pub enum AccessError {
    #[error("Database error: {0}")]
    Database(String),
}

impl From<DbErr> for AccessError {
    fn from(err: DbErr) -> Self {
        AccessError::Database(err.to_string())
    }
}

#[async_trait]
pub trait AccessControl: Send + Sync {
    /// The acting user when `username` may perform `action` on `program_code`; `None` when the
    /// account is unknown / inactive or the permission is missing.
    async fn authorize(
        &self,
        username: &str,
        program_code: &str,
        action: PermissionAction,
    ) -> Result<Option<Actor>, AccessError>;

    /// Like [`authorize`](Self::authorize), but for work on one unit (`target`): a grant limited
    /// to one scope of the group only counts when the unit is inside that scope, and a group-wide
    /// grant only when the unit is inside some scope of the group. `None` = "anywhere". The
    /// default ignores the target so test doubles that only implement `authorize` keep working.
    async fn authorize_in(
        &self,
        username: &str,
        program_code: &str,
        action: PermissionAction,
        _target: Option<ScopeTarget>,
    ) -> Result<Option<Actor>, AccessError> {
        self.authorize(username, program_code, action).await
    }

    /// The [`ScopeTarget`] of a unit (organization / company / branch / department) so a router
    /// can check a write against the unit it touches. `None` when the unit does not exist (the
    /// command then reports the 404) or for test doubles.
    async fn unit_target(&self, _level: ScopeLevel, _unit_id: Uuid) -> Result<Option<ScopeTarget>, AccessError> {
        Ok(None)
    }

    /// Everything `username` may do: program code -> actions (READ / WRITE / APPROVE), for the
    /// web app to hide what the account cannot use. Empty when the account is unknown /
    /// inactive or its group is switched off. The default is "nothing" so test doubles that only
    /// implement [`authorize`](Self::authorize) keep working.
    async fn permissions(
        &self,
        _username: &str,
    ) -> Result<BTreeMap<String, BTreeSet<&'static str>>, AccessError> {
        Ok(BTreeMap::new())
    }
}

/// `AccessControl` backed by `su_user`, `su_permission` and `db_program`. Decisions come from a
/// per-account [`AccessSnapshot`], read through Valkey when a cache is attached.
#[derive(Clone)]
pub struct DbAccessControl {
    db: DatabaseConnection,
    caches: ReadCaches,
}

impl DbAccessControl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db, caches: ReadCaches::disabled() }
    }

    /// Keeps each account's snapshot in Valkey (60 s, cleared on every permission write)
    pub fn with_cache(mut self, caches: ReadCaches) -> Self {
        self.caches = caches;
        self
    }

    /// The account's snapshot: from Valkey, or built from the database and stored
    async fn snapshot(&self, username: &str) -> Result<AccessSnapshot, AccessError> {
        // Accounts are matched by name exactly as the database does; the name is part of the key
        let cache = self.caches.ns(cache_ns::ACL).language_free();
        let key = cache.key("snapshot", &username);
        cache
            .through_key(key, cache_ttl::ACL, || AccessSnapshot::load(&self.db, username))
            .await
    }
}

#[async_trait]
impl AccessControl for DbAccessControl {
    async fn authorize(
        &self,
        username: &str,
        program_code: &str,
        action: PermissionAction,
    ) -> Result<Option<Actor>, AccessError> {
        self.authorize_in(username, program_code, action, None).await
    }

    async fn authorize_in(
        &self,
        username: &str,
        program_code: &str,
        action: PermissionAction,
        target: Option<ScopeTarget>,
    ) -> Result<Option<Actor>, AccessError> {
        Ok(self.snapshot(username).await?.authorize(program_code, action, target.as_ref()))
    }

    async fn unit_target(&self, level: ScopeLevel, unit_id: Uuid) -> Result<Option<ScopeTarget>, AccessError> {
        Ok(match level {
            ScopeLevel::Organization => db_organization::Entity::find_by_id(unit_id)
                .one(&self.db)
                .await?
                .map(|o| ScopeTarget::organization(o.id)),
            ScopeLevel::Company => db_company::Entity::find_by_id(unit_id)
                .one(&self.db)
                .await?
                .map(|c| ScopeTarget::company(c.organization_id, c.id)),
            ScopeLevel::Branch => db_branch::Entity::find_by_id(unit_id)
                .one(&self.db)
                .await?
                .map(|b| ScopeTarget::branch(b.organization_id, b.company_id, b.id)),
            ScopeLevel::Department => db_department::Entity::find_by_id(unit_id)
                .one(&self.db)
                .await?
                .map(|d| ScopeTarget::department(d.organization_id, d.company_id, d.branch_id, d.id)),
        })
    }

    async fn permissions(
        &self,
        username: &str,
    ) -> Result<BTreeMap<String, BTreeSet<&'static str>>, AccessError> {
        Ok(self.snapshot(username).await?.permissions())
    }
}
