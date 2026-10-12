//! Everything the permission checks need to know about one account, loaded once and cached in
//! Valkey (namespace `acl`): the account, the groups it holds and where, and which of those
//! groups grant which action on which program. `authorize_in` and `permissions` then answer from
//! the snapshot with no query at all, instead of the ~6 queries every protected request cost.
//!
//! The snapshot is cleared whenever the data it was built from is written (users, assignments,
//! groups, the permission matrix, programs, suspensions) and expires after a minute regardless, so
//! a revoked right cannot outlive that even if an invalidation is lost.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::entity::sea_orm_active_enums::ScopeLevel;
use domain::entity::{
    db_master_lookup, db_program, su_permission, su_permission_group, su_user, su_user_assignment,
};

use super::{AccessError, Actor, PermissionAction, ScopeTarget, grant_applies};

/// One group the account holds and where: `None` = anywhere (an account with no assignment rows)
pub type Holding = (Uuid, Option<(ScopeLevel, Uuid)>);

/// An active program's grant: `group` may do `action` on `program`
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grant {
    pub program: String,
    /// `READ` / `WRITE` / `APPROVE`
    pub action: String,
    pub group: Uuid,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessSnapshot {
    /// The active account; `None` for an unknown or switched-off one (nothing is allowed)
    pub user: Option<(Uuid, String)>,
    pub holdings: Vec<Holding>,
    /// Ids of the active groups among the holdings (a switched-off group grants nothing)
    pub active_groups: Vec<Uuid>,
    pub grants: Vec<Grant>,
}

impl AccessSnapshot {
    /// Builds the snapshot of `username` from the database
    pub async fn load(db: &DatabaseConnection, username: &str) -> Result<Self, AccessError> {
        let Some(user) = su_user::Entity::find()
            .filter(su_user::Column::Username.eq(username))
            .one(db)
            .await?
            .filter(|u| u.active)
        else {
            return Ok(Self::default());
        };

        let rows = su_user_assignment::Entity::find()
            .filter(su_user_assignment::Column::UserId.eq(user.id))
            .all(db)
            .await?;
        let holdings: Vec<Holding> = if rows.is_empty() {
            // Legacy data: the primary group at any scope
            vec![(user.permission_group_id, None)]
        } else {
            rows.into_iter()
                .filter_map(|r| {
                    let unit = match r.scope_level {
                        ScopeLevel::Organization => r.organization_id,
                        ScopeLevel::Company => r.company_id,
                        ScopeLevel::Branch => r.branch_id,
                        ScopeLevel::Department => r.department_id,
                    }?;
                    Some((r.permission_group_id, Some((r.scope_level, unit))))
                })
                .collect()
        };

        let held: Vec<Uuid> = holdings.iter().map(|(g, _)| *g).collect();
        let active_groups: Vec<Uuid> = su_permission_group::Entity::find()
            .filter(su_permission_group::Column::Id.is_in(held))
            .filter(su_permission_group::Column::Active.eq(true))
            .all(db)
            .await?
            .into_iter()
            .map(|g| g.id)
            .collect();
        if active_groups.is_empty() {
            return Ok(Self {
                user: Some((user.id, user.username)),
                holdings,
                active_groups,
                grants: Vec::new(),
            });
        }

        // lookup id -> action code (the lookup table can repeat rows per code)
        let actions: HashMap<Uuid, &'static str> = db_master_lookup::Entity::find()
            .filter(db_master_lookup::Column::Category.eq("permission_type"))
            .all(db)
            .await?
            .into_iter()
            .filter_map(|l| {
                [PermissionAction::Read, PermissionAction::Write, PermissionAction::Approve]
                    .into_iter()
                    .find(|a| l.code.eq_ignore_ascii_case(a.code()))
                    .map(|a| (l.id, a.code()))
            })
            .collect();
        let programs: HashMap<Uuid, String> = db_program::Entity::find()
            .filter(db_program::Column::Active.eq(true))
            .all(db)
            .await?
            .into_iter()
            .map(|p| (p.id, p.code))
            .collect();

        let grants = su_permission::Entity::find()
            .filter(su_permission::Column::PermissionGroupId.is_in(active_groups.clone()))
            .filter(su_permission::Column::Active.eq(true))
            .all(db)
            .await?
            .into_iter()
            .filter_map(|p| {
                Some(Grant {
                    program: programs.get(&p.program_id)?.clone(),
                    action: (*actions.get(&p.master_lookup_id)?).to_string(),
                    group: p.permission_group_id,
                })
            })
            .collect();
        Ok(Self { user: Some((user.id, user.username)), holdings, active_groups, grants })
    }

    /// The acting user when this account may do `action` on `program_code` (on `target`, when
    /// given: a grant limited to one scope only counts when the unit is inside it)
    pub fn authorize(
        &self,
        program_code: &str,
        action: PermissionAction,
        target: Option<&ScopeTarget>,
    ) -> Option<Actor> {
        let (user_id, username) = self.user.as_ref()?;
        if self.active_groups.is_empty() {
            return None;
        }
        let granting: HashSet<Uuid> = self
            .grants
            .iter()
            .filter(|g| g.program == program_code && g.action.eq_ignore_ascii_case(action.code()))
            .map(|g| g.group)
            .collect();
        let scopes: Vec<Option<(ScopeLevel, Uuid)>> = self
            .holdings
            .iter()
            .filter(|(g, _)| granting.contains(g) && self.active_groups.contains(g))
            .map(|(_, scope)| scope.clone())
            .collect();
        grant_applies(&scopes, target).then(|| Actor { user_id: *user_id, username: username.clone() })
    }

    /// Everything the account may do, anywhere: program code -> actions
    pub fn permissions(&self) -> BTreeMap<String, BTreeSet<&'static str>> {
        let mut out: BTreeMap<String, BTreeSet<&'static str>> = BTreeMap::new();
        if self.user.is_none() {
            return out;
        }
        for grant in &self.grants {
            if let Some(action) = [PermissionAction::Read, PermissionAction::Write, PermissionAction::Approve]
                .into_iter()
                .find(|a| grant.action.eq_ignore_ascii_case(a.code()))
            {
                out.entry(grant.program.clone()).or_default().insert(action.code());
            }
        }
        out
    }
}
