//! DBPG003 query: one page of employees matching the filter

use application_core::view_scope;
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use super::like_search::{contains_ci, escape_like};
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::{
    DEFAULT_PAGE_SIZE, EmployeeFilterQuery, EmployeePageDto, MAX_PAGE_SIZE, db_employee,
    db_person_name_label,
};
use crate::features::db::dbpg003::queries::db_dbpg003_queries::DbDbpg003Queries;
use crate::features::db::view_scope_filters::branch_data_condition;

impl DbDbpg003Queries {
    pub(super) async fn list_employees(
        &self,
        filter: EmployeeFilterQuery,
    ) -> Result<EmployeePageDto, Dbpg003Error> {
        let mut cond = Condition::all();
        // What the request may look at comes first; the filters below can only narrow it further
        if let Some(companies) = view_scope::companies() {
            cond = cond.add(db_employee::Column::CompanyId.is_in(companies));
        }
        if let Some(visible) = branch_data_condition(db_employee::Column::BranchId, db_employee::Column::DepartmentId) {
            cond = cond.add(visible);
        }
        if let Some(v) = filter.company_id {
            cond = cond.add(db_employee::Column::CompanyId.eq(v));
        }
        if let Some(v) = filter.branch_id {
            cond = cond.add(db_employee::Column::BranchId.eq(v));
        }
        if let Some(v) = filter.department_id {
            cond = cond.add(db_employee::Column::DepartmentId.eq(v));
        }
        if let Some(v) = filter.active {
            cond = cond.add(db_employee::Column::Active.eq(v));
        }

        let term = filter.search.as_deref().unwrap_or("").trim().to_lowercase();
        if !term.is_empty() {
            let like = format!("%{}%", escape_like(&term));
            let person_ids: Vec<Uuid> = db_person_name_label::Entity::find()
                .filter(
                    Condition::any()
                        .add(contains_ci(db_person_name_label::Column::FirstName, &like))
                        .add(contains_ci(db_person_name_label::Column::LastName, &like))
                        .add(contains_ci(db_person_name_label::Column::Nickname, &like)),
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

        let limit = Ord::clamp(filter.limit.unwrap_or(DEFAULT_PAGE_SIZE), 1, MAX_PAGE_SIZE);
        let page = Ord::max(filter.page.unwrap_or(1), 1);

        let paginator = db_employee::Entity::find()
            .filter(cond)
            .order_by_asc(db_employee::Column::Code)
            .paginate(&self.db, limit);
        let total = paginator.num_items().await?;
        let employees = paginator.fetch_page(page - 1).await?;

        Ok(EmployeePageDto {
            items: self.employee_dtos(employees).await?,
            total,
            page,
            limit,
        })
    }
}
