//! DBPG002 query: departments with optional company / branch / search / active filters

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{DepartmentDto, DepartmentFilterQuery, db_department};
use crate::features::db::dbpg002::queries::db_dbpg002_queries::DbDbpg002Queries;
use crate::features::db::view_scope_filters::{by_branch_data, by_company};

impl DbDbpg002Queries {
    pub(super) async fn list_departments(
        &self,
        filter: DepartmentFilterQuery,
    ) -> Result<Vec<DepartmentDto>, Dbpg002Error> {
        let mut select = by_branch_data(
            by_company(db_department::Entity::find(), db_department::Column::CompanyId),
            db_department::Column::BranchId,
            db_department::Column::Id,
        );
        if let Some(company_id) = filter.company_id {
            select = select.filter(db_department::Column::CompanyId.eq(company_id));
        }
        if let Some(branch_id) = filter.branch_id {
            select = select.filter(db_department::Column::BranchId.eq(branch_id));
        }
        if let Some(active) = filter.active {
            select = select.filter(db_department::Column::Active.eq(active));
        }
        let departments = select
            .order_by_asc(db_department::Column::Code)
            .all(&self.db)
            .await?;
        self.department_dtos(departments, filter.search.as_deref())
            .await
    }
}
