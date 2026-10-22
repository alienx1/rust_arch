//! A department's default permission group.
//!
//! Every account of the department's employees holds that group at the department's **branch**
//! (scope level `branch`, the branch the department belongs to), added automatically as an
//! ordinary `su_user_assignment` row: when an account is created, and when the default is set or
//! changed on the department (for the accounts that exist then). The row is the account's own from
//! then on: SUPG004 shows it, and it can be edited or removed there. Nothing is taken away when
//! the default changes, and an employee who moves to another department keeps what they have.

use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, Set};
use std::collections::HashSet;
use uuid::Uuid;

use domain::entity::sea_orm_active_enums::ScopeLevel;
use domain::entity::{db_department, db_employee, su_user, su_user_assignment};

/// A permission group at one scope unit: `(group, level, unit)`
pub type Assignment = (Uuid, ScopeLevel, Uuid);

/// What the department gives its accounts: its default group at its branch (`None` without one)
pub fn default_assignment(department: &db_department::Model) -> Option<Assignment> {
    department.default_permission_group_id.map(|group| (group, ScopeLevel::Branch, department.branch_id))
}

/// `assignments` plus the department's default, unless the account already holds that group
/// somewhere (a broader or other scope is the admin's decision and stays). The first assignment
/// stays first: its group is the account's primary group, so an account created with no pick at
/// all gets the default as its primary.
pub fn with_department_default(mut assignments: Vec<Assignment>, default: Option<Assignment>) -> Vec<Assignment> {
    if let Some(default) = default
        && !assignments.iter().any(|(group, _, _)| *group == default.0)
    {
        assignments.push(default);
    }
    assignments
}

/// Gives the department's default group to the accounts of its employees that do not hold it yet.
/// Returns how many accounts got it. Runs inside the caller's connection / transaction.
pub async fn add_to_existing_accounts(
    conn: &impl ConnectionTrait,
    department: &db_department::Model,
    actor_id: Option<Uuid>,
) -> Result<u64, DbErr> {
    let Some((group_id, level, unit_id)) = default_assignment(department) else { return Ok(0) };

    let employees: Vec<Uuid> = db_employee::Entity::find()
        .filter(db_employee::Column::DepartmentId.eq(department.id))
        .all(conn)
        .await?
        .into_iter()
        .map(|e| e.id)
        .collect();
    if employees.is_empty() {
        return Ok(0);
    }
    let users: Vec<Uuid> = su_user::Entity::find()
        .filter(su_user::Column::EmployeeId.is_in(employees))
        .all(conn)
        .await?
        .into_iter()
        .map(|u| u.id)
        .collect();
    if users.is_empty() {
        return Ok(0);
    }
    let holders: HashSet<Uuid> = su_user_assignment::Entity::find()
        .filter(su_user_assignment::Column::UserId.is_in(users.clone()))
        .filter(su_user_assignment::Column::PermissionGroupId.eq(group_id))
        .all(conn)
        .await?
        .into_iter()
        .map(|a| a.user_id)
        .collect();

    let mut added = 0;
    for user_id in users.into_iter().filter(|u| !holders.contains(u)) {
        su_user_assignment::ActiveModel {
            id: Set(Uuid::now_v7()),
            user_id: Set(user_id),
            permission_group_id: Set(group_id),
            scope_level: Set(level.clone()),
            organization_id: Set(None),
            company_id: Set(None),
            branch_id: Set(Some(unit_id)),
            department_id: Set(None),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
        .insert(conn)
        .await?;
        added += 1;
    }
    Ok(added)
}
