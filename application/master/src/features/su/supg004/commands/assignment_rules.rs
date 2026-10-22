//! SUPG004 assignment rules shared by create and update: validate the picked (group, unit) pairs
//! and keep the `su_user_assignment` rows of an account in step with them.

use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::supg004_executor::Supg004CommandExecutor;
use crate::features::su::department_default_group::default_assignment;
use crate::features::su::supg004::errors::Supg004Error;
use crate::features::su::supg004::models::AssignmentCommand;
use domain::entity::sea_orm_active_enums::ScopeLevel;
use domain::entity::{
    db_branch, db_company, db_department, db_organization, su_user_assignment,
};

/// A validated assignment: group, scope level and unit
pub type Assignment = (Uuid, ScopeLevel, Uuid);

pub fn scope_level_of(raw: &str) -> Result<ScopeLevel, Supg004Error> {
    match raw.trim().to_lowercase().as_str() {
        "organization" => Ok(ScopeLevel::Organization),
        "company" => Ok(ScopeLevel::Company),
        "branch" => Ok(ScopeLevel::Branch),
        "department" => Ok(ScopeLevel::Department),
        other => Err(Supg004Error::Validation(format!(
            "Scope level '{other}' must be organization, company, branch or department"
        ))),
    }
}

/// At least one assignment, a known scope level on each; repeats of the same (group, unit) collapse
/// into one. The first assignment stays first: its group is the account's primary group.
pub fn normalize_assignments(cmds: &[AssignmentCommand]) -> Result<Vec<Assignment>, Supg004Error> {
    if cmds.is_empty() {
        return Err(Supg004Error::Validation("Pick at least one assignment".to_string()));
    }
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for c in cmds {
        let level = scope_level_of(&c.scope_level)?;
        if seen.insert((c.permission_group_id, c.scope_id)) {
            out.push((c.permission_group_id, level, c.scope_id));
        }
    }
    Ok(out)
}

fn unit_columns(level: &ScopeLevel, id: Uuid) -> [Option<Uuid>; 4] {
    match level {
        ScopeLevel::Organization => [Some(id), None, None, None],
        ScopeLevel::Company => [None, Some(id), None, None],
        ScopeLevel::Branch => [None, None, Some(id), None],
        ScopeLevel::Department => [None, None, None, Some(id)],
    }
}

impl Supg004CommandExecutor {
    /// Validates the picks and returns them with the code of every group involved. Every group
    /// must exist and be active, every unit must exist.
    pub(super) async fn validate_assignments(
        &self,
        cmds: &[AssignmentCommand],
    ) -> Result<(Vec<Assignment>, HashMap<Uuid, String>), Supg004Error> {
        let assignments = normalize_assignments(cmds)?;
        let mut groups: HashMap<Uuid, String> = HashMap::new();
        for (group_id, level, unit_id) in &assignments {
            if !groups.contains_key(group_id) {
                let group = self.group(*group_id).await?;
                if !group.active {
                    return Err(Supg004Error::Validation(format!(
                        "The permission group '{}' is switched off",
                        group.code
                    )));
                }
                groups.insert(group.id, group.code);
            }
            let exists = match level {
                ScopeLevel::Organization => db_organization::Entity::find_by_id(*unit_id).one(&self.db).await?.is_some(),
                ScopeLevel::Company => db_company::Entity::find_by_id(*unit_id).one(&self.db).await?.is_some(),
                ScopeLevel::Branch => db_branch::Entity::find_by_id(*unit_id).one(&self.db).await?.is_some(),
                ScopeLevel::Department => db_department::Entity::find_by_id(*unit_id).one(&self.db).await?.is_some(),
            };
            if !exists {
                return Err(Supg004Error::NotFound(format!("The scope unit '{unit_id}' does not exist")));
            }
        }
        Ok((assignments, groups))
    }

    /// What the employee's department gives its accounts: its default permission group at its
    /// branch. `None` when the department has none, or the group is switched off now.
    pub(super) async fn department_default(&self, department_id: Uuid) -> Result<Option<Assignment>, Supg004Error> {
        let Some(department) = db_department::Entity::find_by_id(department_id).one(&self.db).await? else {
            return Ok(None);
        };
        let Some(default) = default_assignment(&department) else { return Ok(None) };
        let active = self.group(default.0).await.map(|g| g.active).unwrap_or(false);
        Ok(active.then_some(default))
    }

    /// The existing assignment rows of an account as (row, (group, level, unit))
    pub(super) async fn assignment_rows(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<(su_user_assignment::Model, Assignment)>, Supg004Error> {
        let rows = su_user_assignment::Entity::find()
            .filter(su_user_assignment::Column::UserId.eq(user_id))
            .all(&self.db)
            .await?;
        Ok(rows
            .into_iter()
            .filter_map(|r| {
                let unit = r.organization_id.or(r.company_id).or(r.branch_id).or(r.department_id)?;
                let key = (r.permission_group_id, r.scope_level.clone(), unit);
                Some((r, key))
            })
            .collect())
    }

    /// Makes the account's rows match `wanted` inside the caller's transaction: rows of unchanged
    /// (group, unit) pairs stay, removed ones are deleted, new ones are inserted.
    pub(super) async fn write_assignments(
        &self,
        conn: &impl ConnectionTrait,
        user_id: Uuid,
        wanted: &[Assignment],
        actor_id: Option<Uuid>,
    ) -> Result<(), Supg004Error> {
        let existing = su_user_assignment::Entity::find()
            .filter(su_user_assignment::Column::UserId.eq(user_id))
            .all(conn)
            .await?;
        let key_of = |r: &su_user_assignment::Model| {
            r.organization_id
                .or(r.company_id)
                .or(r.branch_id)
                .or(r.department_id)
                .map(|u| (r.permission_group_id, u))
        };
        let wanted_keys: HashSet<(Uuid, Uuid)> = wanted.iter().map(|(g, _, u)| (*g, *u)).collect();
        let kept: HashSet<(Uuid, Uuid)> = existing.iter().filter_map(key_of).filter(|k| wanted_keys.contains(k)).collect();

        let removed: Vec<Uuid> = existing
            .iter()
            .filter(|r| key_of(r).is_none_or(|k| !wanted_keys.contains(&k)))
            .map(|r| r.id)
            .collect();
        if !removed.is_empty() {
            su_user_assignment::Entity::delete_many()
                .filter(su_user_assignment::Column::Id.is_in(removed))
                .exec(conn)
                .await?;
        }
        for (group_id, level, unit_id) in wanted.iter().filter(|(g, _, u)| !kept.contains(&(*g, *u))) {
            let [org, company, branch, department] = unit_columns(level, *unit_id);
            su_user_assignment::ActiveModel {
                id: Set(Uuid::now_v7()),
                user_id: Set(user_id),
                permission_group_id: Set(*group_id),
                scope_level: Set(level.clone()),
                organization_id: Set(org),
                company_id: Set(company),
                branch_id: Set(branch),
                department_id: Set(department),
                change_by: Set(actor_id),
                row_version: Set(1),
            }
            .insert(conn)
            .await?;
        }
        Ok(())
    }
}
