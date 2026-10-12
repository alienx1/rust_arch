//! SUPG003 save-matrix action: validates the desired grants and applies only the difference to
//! the group's `su_permission` rows, in one transaction.

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use std::collections::{HashMap, HashSet};
use tracing::info;
use uuid::Uuid;

use super::supg003_executor::Supg003CommandExecutor;
use crate::features::su::supg003::errors::Supg003Error;
use crate::features::su::supg003::models::{
    MatrixAction, PERMISSION_PROGRAM_CODE, PermissionMatrixDto, SaveMatrixCommand, is_admin_group,
    su_permission, su_permission_group,
};
use crate::features::su::supg003::queries::group_user_ids;
use domain::entity::{db_master_lookup, db_program};

/// One existing `su_permission` row of the group
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExistingGrant {
    pub row_id: Uuid,
    pub program_id: Uuid,
    pub action: MatrixAction,
    pub active: bool,
}

/// What saving a matrix has to do
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ChangePlan {
    /// Rows that exist but are switched off, to be switched on
    pub activate: Vec<Uuid>,
    /// Active rows no longer wanted, to be switched off
    pub deactivate: Vec<Uuid>,
    /// Wanted grants without any row yet
    pub insert: Vec<(Uuid, MatrixAction)>,
}

/// Diff of the desired grant set against the group's existing rows. A lookup table that repeats
/// a code can leave several rows for one grant: any active one keeps it, and removing it switches
/// all of them off.
pub fn plan_changes(existing: &[ExistingGrant], desired: &HashSet<(Uuid, MatrixAction)>) -> ChangePlan {
    let mut plan = ChangePlan::default();
    let mut rows_of: HashMap<(Uuid, MatrixAction), Vec<&ExistingGrant>> = HashMap::new();
    for e in existing {
        rows_of.entry((e.program_id, e.action)).or_default().push(e);
    }

    for (key, rows) in &rows_of {
        let any_active = rows.iter().any(|r| r.active);
        if desired.contains(key) {
            if !any_active {
                // Switch on a single row; the rest stay off
                plan.activate.push(rows[0].row_id);
            }
        } else {
            plan.deactivate.extend(rows.iter().filter(|r| r.active).map(|r| r.row_id));
        }
    }
    for key in desired {
        if !rows_of.contains_key(key) {
            plan.insert.push(*key);
        }
    }
    plan.activate.sort();
    plan.deactivate.sort();
    plan.insert.sort_by_key(|(p, a)| (*p, a.code()));
    plan
}

/// `true` when `desired` still gives READ and WRITE on the permission-administration program, so
/// a group editing its own matrix cannot lock itself (and its users) out of this page.
pub fn keeps_own_access(desired: &HashSet<(Uuid, MatrixAction)>, permission_program: Uuid) -> bool {
    desired.contains(&(permission_program, MatrixAction::Read))
        && desired.contains(&(permission_program, MatrixAction::Write))
}

impl Supg003CommandExecutor {
    pub(super) async fn save_matrix(
        &self,
        id: Uuid,
        cmd: SaveMatrixCommand,
        actor_id: Option<Uuid>,
    ) -> Result<PermissionMatrixDto, Supg003Error> {
        let group = su_permission_group::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg003Error::NotFound(id.to_string()))?;
        if is_admin_group(&group.code) {
            return Err(Supg003Error::Validation(
                "The ADMIN group has full access to every program and its matrix cannot be changed".to_string(),
            ));
        }

        // Every program id must exist; a grant needs the program's system and module for its row
        let programs: HashMap<Uuid, db_program::Model> = db_program::Entity::find()
            .all(&self.db)
            .await?
            .into_iter()
            .map(|p| (p.id, p))
            .collect();
        let mut desired: HashSet<(Uuid, MatrixAction)> = HashSet::new();
        for g in &cmd.grants {
            if !programs.contains_key(&g.program_id) {
                return Err(Supg003Error::NotFound(format!("Program '{}' does not exist", g.program_id)));
            }
            for (on, action) in [
                (g.read, MatrixAction::Read),
                (g.write, MatrixAction::Write),
                (g.approve, MatrixAction::Approve),
            ] {
                if on {
                    desired.insert((g.program_id, action));
                }
            }
        }
        // Action of each lookup row, and the canonical (lowest id) row per action for inserts
        let mut lookups = db_master_lookup::Entity::find()
            .filter(db_master_lookup::Column::Category.eq("permission_type"))
            .all(&self.db)
            .await?;
        lookups.sort_by_key(|l| l.id);
        let mut action_of: HashMap<Uuid, MatrixAction> = HashMap::new();
        let mut canonical: HashMap<MatrixAction, Uuid> = HashMap::new();
        for l in &lookups {
            if let Some(a) = MatrixAction::from_code(&l.code) {
                action_of.insert(l.id, a);
                canonical.entry(a).or_insert(l.id);
            }
        }
        if MatrixAction::ALL.iter().any(|a| !canonical.contains_key(a)) {
            return Err(Supg003Error::Internal(
                "The permission_type lookup is missing READ, WRITE or APPROVE".to_string(),
            ));
        }

        let existing_rows = su_permission::Entity::find()
            .filter(su_permission::Column::PermissionGroupId.eq(id))
            .all(&self.db)
            .await?;

        // WRITE / APPROVE without READ would grant a page the menu then hides: require READ
        if let Some((p, a)) = desired.iter().find(|(p, a)| {
            *a != MatrixAction::Read && !desired.contains(&(*p, MatrixAction::Read))
        }) {
            return Err(Supg003Error::Validation(format!(
                "{} on '{}' needs READ on the same program",
                a.code(),
                programs[p].code
            )));
        }

        // A user editing their own group's matrix must keep access to this page
        if let Some(actor) = actor_id {
            if group_user_ids(&self.db, id).await?.contains(&actor) {
                let permission_program = programs
                    .values()
                    .find(|p| p.code.eq_ignore_ascii_case(PERMISSION_PROGRAM_CODE))
                    .map(|p| p.id);
                if let Some(pp) = permission_program {
                    if !keeps_own_access(&desired, pp) {
                        return Err(Supg003Error::Validation(format!(
                            "You belong to this group: it must keep READ and WRITE on {PERMISSION_PROGRAM_CODE}, or you would lose access to this page"
                        )));
                    }
                }
            }
        }

        let existing: Vec<ExistingGrant> = existing_rows
            .iter()
            .filter_map(|r| {
                action_of.get(&r.master_lookup_id).map(|a| ExistingGrant {
                    row_id: r.id,
                    program_id: r.program_id,
                    action: *a,
                    active: r.active,
                })
            })
            .collect();
        let plan = plan_changes(&existing, &desired);
        info!(
            "Saving matrix of group {id}: +{} activate, -{} deactivate, {} new",
            plan.activate.len(),
            plan.deactivate.len(),
            plan.insert.len()
        );

        let txn = self.db.begin().await?;
        let by_id: HashMap<Uuid, &su_permission::Model> = existing_rows.iter().map(|r| (r.id, r)).collect();
        for (ids, active) in [(&plan.activate, true), (&plan.deactivate, false)] {
            for row_id in ids {
                let row = by_id[row_id];
                let mut am: su_permission::ActiveModel = (*row).clone().into();
                am.active = Set(active);
                am.change_by = Set(actor_id);
                am.row_version = Set(row.row_version + 1);
                am.update(&txn).await?;
            }
        }
        for (program_id, action) in &plan.insert {
            let program = &programs[program_id];
            su_permission::ActiveModel {
                id: Set(Uuid::now_v7()),
                permission_group_id: Set(id),
                system_id: Set(program.system_id),
                module_id: Set(program.module_id),
                program_id: Set(*program_id),
                master_lookup_id: Set(canonical[action]),
                active: Set(true),
                change_by: Set(actor_id),
                row_version: Set(1),
            }
            .insert(&txn)
            .await?;
        }
        txn.commit().await?;

        self.queries
            .group_matrix(id)
            .await?
            .ok_or_else(|| Supg003Error::Internal("Failed to retrieve the saved matrix".to_string()))
    }
}
