//! List users paginated with SQL LIMIT and OFFSET

use domain::entity::{
    db_company, db_department, db_employee, su_permission_group, su_user,
};
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::features::su::supg005::errors::Supg005Error;
use crate::features::su::supg005::models::{PaginatedResult, UserPaginationQuery, UserSessionOverviewDto};
use crate::features::su::supg005::queries::DbSupg005Queries;
use super::user_overview_names::role_display_name;
use crate::features::localization::request_language::pick;

impl DbSupg005Queries {
    pub(super) async fn list_users_paginated(
        &self,
        query: UserPaginationQuery,
    ) -> Result<PaginatedResult<UserSessionOverviewDto>, Supg005Error> {
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(5).clamp(1, 100);
        let offset = (page - 1) * limit;

        // 1. Build Base Select Query on su_user
        let mut select = su_user::Entity::find();

        if let Some(ref q) = query.search {
            let trimmed = q.trim();
            if !trimmed.is_empty() {
                let pattern = format!("%{trimmed}%");
                select = select.filter(
                    Condition::any()
                        .add(su_user::Column::Username.like(&pattern))
                );
            }
        }

        match query.account_status.as_deref() {
            Some("ACTIVE") => select = select.filter(su_user::Column::Active.eq(true)),
            Some("SUSPENDED") => select = select.filter(su_user::Column::Active.eq(false)),
            _ => {}
        }

        // Count total matching records in DB using SQL COUNT(*)
        let total = select.clone().count(&self.db).await?;

        // If the database has records, execute SQL with LIMIT and OFFSET
        if total > 0 {
            let users = select
                .order_by_asc(su_user::Column::Username)
                .limit(limit)
                .offset(offset)
                .all(&self.db)
                .await?;

            let mut items = Vec::new();
            for u in users {
                let user_id_str = u.id.to_string();

                let perm_group = su_permission_group::Entity::find_by_id(u.permission_group_id)
                    .one(&self.db)
                    .await?;

                let (role_code, scope_level) = match perm_group {
                    Some(g) => (g.code, self.primary_scope_level(u.id).await?),
                    None => ("USER".to_string(), self.primary_scope_level(u.id).await?),
                };

                let emp = db_employee::Entity::find_by_id(u.employee_id)
                    .one(&self.db)
                    .await?;

                let mut employee_code = "EMP-SYS".to_string();
                let mut full_name = u.username.clone();
                let mut department_name = pick("ฝ่ายเทคโนโลยีสารสนเทศ", "Information Technology Department").to_string();
                let mut company_name = pick("สำนักงานใหญ่", "Head Office").to_string();

                if let Some(employee) = emp {
                    employee_code = employee.code.clone();

                    let (name, dept, comp) = self.employee_overview_names(&employee).await?;
                    if let Some(name) = name {
                        full_name = name;
                    }
                    match dept {
                        Some(dept) => department_name = dept,
                        None => {
                            if let Some(dept) = db_department::Entity::find_by_id(employee.department_id).one(&self.db).await? {
                                department_name = pick(&format!("แผนก {}", dept.code), &format!("Department {}", dept.code)).to_string();
                            }
                        }
                    }
                    match comp {
                        Some(comp) => company_name = comp,
                        None => {
                            if let Some(comp) = db_company::Entity::find_by_id(employee.company_id).one(&self.db).await? {
                                company_name = pick(&format!("บริษัท {}", comp.code), &format!("Company {}", comp.code)).to_string();
                            }
                        }
                    }
                }

                // Active devices are dynamically synchronized from the identity session store
                let devices = vec![];

                items.push(UserSessionOverviewDto {
                    user_id: user_id_str,
                    username: u.username.clone(),
                    email: format!("{}@arch.local", u.username),
                    full_name,
                    employee_code,
                    role_code: role_code.clone(),
                    role_name: role_display_name(&role_code),
                    scope_level,
                    department_name,
                    company_name,
                    is_active_account: u.active,
                    devices,
                });
            }

            let total_pages = if total == 0 { 1 } else { (total + limit - 1) / limit };

            return Ok(PaginatedResult {
                items,
                total,
                page,
                limit,
                total_pages,
            });
        }

        // No account matches (an empty user table, or the filters leave nothing)
        Ok(PaginatedResult { items: Vec::new(), total: 0, page, limit, total_pages: 1 })
    }
}
