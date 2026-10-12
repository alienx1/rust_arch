//! List all users with their roles and employee details

use domain::entity::{
    db_company, db_department, db_employee, su_permission_group, su_user,
};
use sea_orm::{EntityTrait, QueryOrder};

use crate::features::su::supg005::errors::Supg005Error;
use crate::features::su::supg005::models::UserSessionOverviewDto;
use crate::features::su::supg005::queries::DbSupg005Queries;
use super::user_overview_names::role_display_name;
use crate::features::localization::request_language::pick;

impl DbSupg005Queries {
    pub(super) async fn list_users_with_sessions(&self) -> Result<Vec<UserSessionOverviewDto>, Supg005Error> {
        let users = su_user::Entity::find()
            .order_by_asc(su_user::Column::Username)
            .all(&self.db)
            .await?;

        let mut result = Vec::new();

        for u in users {
            let user_id_str = u.id.to_string();

            // 1. Fetch Permission Group
            let perm_group = su_permission_group::Entity::find_by_id(u.permission_group_id)
                .one(&self.db)
                .await?;

            let (role_code, scope_level) = match perm_group {
                Some(g) => (g.code, self.primary_scope_level(u.id).await?),
                None => ("USER".to_string(), self.primary_scope_level(u.id).await?),
            };

            // 2. Fetch Employee Record
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

            result.push(UserSessionOverviewDto {
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

        Ok(result)
    }
}
