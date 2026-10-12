//! SUPG004 employee-options query: active employees without an account, matching the search.

use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use uuid::Uuid;

use super::db_supg004_queries::DbSupg004Queries;
use crate::features::db::dbpg002::queries::db_dbpg002_queries::fold_names;
use crate::features::db::dbpg003::queries::db_dbpg003_queries::{contains_ci, escape_like};
use crate::features::localization::request_language::pick;
use crate::features::su::supg004::errors::Supg004Error;
use crate::features::su::supg004::models::{
    EMPLOYEE_OPTIONS_LIMIT, EmployeeOptionDto, EmployeeOptionsQuery, su_user,
};
use domain::entity::{db_company_label, db_department_label, db_employee, db_person_name_label};

impl DbSupg004Queries {
    pub(super) async fn employee_options(&self, query: EmployeeOptionsQuery) -> Result<Vec<EmployeeOptionDto>, Supg004Error> {
        let with_account: Vec<Uuid> = su_user::Entity::find()
            .select_only()
            .column(su_user::Column::EmployeeId)
            .into_tuple()
            .all(&self.db)
            .await?;

        let mut cond = Condition::all().add(db_employee::Column::Active.eq(true));
        if !with_account.is_empty() {
            cond = cond.add(db_employee::Column::Id.is_not_in(with_account));
        }
        let term = query.search.as_deref().unwrap_or("").trim().to_lowercase();
        if !term.is_empty() {
            let like = format!("%{}%", escape_like(&term));
            let person_ids: Vec<Uuid> = db_person_name_label::Entity::find()
                .filter(
                    Condition::any()
                        .add(contains_ci(db_person_name_label::Column::FirstName, &like))
                        .add(contains_ci(db_person_name_label::Column::LastName, &like)),
                )
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| l.person_name_id)
                .collect();
            cond = cond.add(
                Condition::any()
                    .add(contains_ci(db_employee::Column::Code, &like))
                    .add(db_employee::Column::PersonNameId.is_in(person_ids)),
            );
        }
        let employees = db_employee::Entity::find()
            .filter(cond)
            .order_by_asc(db_employee::Column::Code)
            .limit(EMPLOYEE_OPTIONS_LIMIT)
            .all(&self.db)
            .await?;

        let names = self.full_names(&employees).await?;
        let department_names = fold_names(
            db_department_label::Entity::find()
                .filter(db_department_label::Column::DepartmentId.is_in(employees.iter().map(|e| e.department_id).collect::<Vec<_>>()))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.department_id, l.language_id, l.label)),
        );
        let company_names = fold_names(
            db_company_label::Entity::find()
                .filter(db_company_label::Column::CompanyId.is_in(employees.iter().map(|e| e.company_id).collect::<Vec<_>>()))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.company_id, l.language_id, l.label)),
        );

        Ok(employees
            .into_iter()
            .map(|e| EmployeeOptionDto {
                full_name: names.get(&e.id).cloned().unwrap_or_default(),
                department_name: department_names.get(&e.department_id).map(|n| pick(&n.0, &n.1).to_string()).unwrap_or_default(),
                company_name: company_names.get(&e.company_id).map(|n| pick(&n.0, &n.1).to_string()).unwrap_or_default(),
                id: e.id,
                code: e.code,
            })
            .collect())
    }
}
