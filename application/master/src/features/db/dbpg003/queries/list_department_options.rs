//! DBPG003 query: all departments (filtered client-side by branch)

use sea_orm::{EntityTrait, QueryOrder};

use crate::features::db::dbpg002::models::{db_department, db_department_label};
use crate::features::db::dbpg002::queries::db_dbpg002_queries::fold_names;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::DepartmentOptionDto;
use crate::features::db::dbpg003::queries::db_dbpg003_queries::DbDbpg003Queries;
use crate::features::db::view_scope_filters::{by_branch_data, by_company};

impl DbDbpg003Queries {
    pub(super) async fn list_department_options(
        &self,
    ) -> Result<Vec<DepartmentOptionDto>, Dbpg003Error> {
        let departments = by_branch_data(
            by_company(db_department::Entity::find(), db_department::Column::CompanyId),
            db_department::Column::BranchId,
            db_department::Column::Id,
        )
            .order_by_asc(db_department::Column::Code)
            .all(&self.db)
            .await?;
        let names = fold_names(
            db_department_label::Entity::find()
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.department_id, l.language_id, l.label)),
        );
        Ok(departments
            .into_iter()
            .map(|d| {
                let (name_th, name_en) = names.get(&d.id).cloned().unwrap_or_default();
                DepartmentOptionDto {
                    id: d.id,
                    company_id: d.company_id,
                    branch_id: d.branch_id,
                    code: d.code,
                    name_th,
                    name_en,
                    active: d.active,
                }
            })
            .collect())
    }
}
