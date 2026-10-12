//! SUPG004 account DTO assembly: employee, department, company and group names per account.

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::db_supg004_queries::DbSupg004Queries;
use super::scope_names::{scope_level_name, scope_path_of};
use crate::features::db::dbpg002::queries::db_dbpg002_queries::fold_names;
use crate::features::localization::request_language::pick;
use crate::features::su::supg004::errors::Supg004Error;
use crate::features::su::supg004::models::{UserAccountDto, UserAssignmentDto, su_user};
use domain::entity::{db_company_label, db_department_label, db_employee, su_permission_group, su_user_assignment};

impl DbSupg004Queries {
    pub(super) async fn user_dtos(&self, users: Vec<su_user::Model>) -> Result<Vec<UserAccountDto>, Supg004Error> {
        if users.is_empty() {
            return Ok(Vec::new());
        }
        let employee_ids: Vec<Uuid> = users.iter().map(|u| u.employee_id).collect::<HashSet<_>>().into_iter().collect();
        let employees = db_employee::Entity::find()
            .filter(db_employee::Column::Id.is_in(employee_ids))
            .all(&self.db)
            .await?;
        let names = self.full_names(&employees).await?;
        let by_id: HashMap<Uuid, &db_employee::Model> = employees.iter().map(|e| (e.id, e)).collect();

        let department_ids: Vec<Uuid> = employees.iter().map(|e| e.department_id).collect::<HashSet<_>>().into_iter().collect();
        let company_ids: Vec<Uuid> = employees.iter().map(|e| e.company_id).collect::<HashSet<_>>().into_iter().collect();
        let department_names = fold_names(
            db_department_label::Entity::find()
                .filter(db_department_label::Column::DepartmentId.is_in(department_ids))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.department_id, l.language_id, l.label)),
        );
        let company_names = fold_names(
            db_company_label::Entity::find()
                .filter(db_company_label::Column::CompanyId.is_in(company_ids))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.company_id, l.language_id, l.label)),
        );
        let group_ids: Vec<Uuid> = users.iter().map(|u| u.permission_group_id).collect::<HashSet<_>>().into_iter().collect();
        let groups: HashMap<Uuid, String> = su_permission_group::Entity::find()
            .filter(su_permission_group::Column::Id.is_in(group_ids))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|g| (g.id, g.code))
            .collect();

        // Assignments per account, the primary group first, then by group code and unit path
        let user_ids: Vec<Uuid> = users.iter().map(|u| u.id).collect();
        let assignment_rows = su_user_assignment::Entity::find()
            .filter(su_user_assignment::Column::UserId.is_in(user_ids))
            .all(&self.db)
            .await?;
        let mut assignments: HashMap<Uuid, Vec<UserAssignmentDto>> = HashMap::new();
        if !assignment_rows.is_empty() {
            let scopes = self.scope_names().await?;
            let parents = self.scope_parents().await?;
            let assigned_group_ids: Vec<Uuid> = assignment_rows.iter().map(|a| a.permission_group_id).collect();
            let assigned_groups: HashMap<Uuid, String> = su_permission_group::Entity::find()
                .filter(su_permission_group::Column::Id.is_in(assigned_group_ids))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|g| (g.id, g.code))
                .collect();
            for a in assignment_rows {
                let unit = a.organization_id.or(a.company_id).or(a.branch_id).or(a.department_id);
                let Some(scope_id) = unit else { continue };
                let (scope_code, th, en) = scopes.get(&scope_id).cloned().unwrap_or_default();
                assignments.entry(a.user_id).or_default().push(UserAssignmentDto {
                    group_id: a.permission_group_id,
                    group_code: assigned_groups.get(&a.permission_group_id).cloned().unwrap_or_default(),
                    scope_level: scope_level_name(&a.scope_level).to_string(),
                    scope_id,
                    scope_code,
                    scope_name: pick(&th, &en).to_string(),
                    scope_path: scope_path_of(Some(scope_id), &scopes, &parents),
                });
            }
            for u in &users {
                if let Some(list) = assignments.get_mut(&u.id) {
                    list.sort_by(|a, b| {
                        (b.group_id == u.permission_group_id)
                            .cmp(&(a.group_id == u.permission_group_id))
                            .then_with(|| a.group_code.cmp(&b.group_code))
                            .then_with(|| a.scope_path.cmp(&b.scope_path))
                    });
                }
            }
        }

        Ok(users
            .into_iter()
            .map(|u| {
                let employee = by_id.get(&u.employee_id);
                UserAccountDto {
                    id: u.id,
                    employee_id: u.employee_id,
                    employee_code: employee.map(|e| e.code.clone()).unwrap_or_default(),
                    full_name: names.get(&u.employee_id).cloned().unwrap_or_default(),
                    department_name: employee
                        .and_then(|e| department_names.get(&e.department_id))
                        .map(|n| pick(&n.0, &n.1).to_string())
                        .unwrap_or_default(),
                    company_name: employee
                        .and_then(|e| company_names.get(&e.company_id))
                        .map(|n| pick(&n.0, &n.1).to_string())
                        .unwrap_or_default(),
                    group_code: groups.get(&u.permission_group_id).cloned().unwrap_or_default(),
                    group_id: u.permission_group_id,
                    assignments: assignments.remove(&u.id).unwrap_or_default(),
                    username: u.username,
                    active: u.active,
                    created_time: u.created_time.to_rfc3339(),
                    row_version: u.row_version,
                }
            })
            .collect())
    }
}
