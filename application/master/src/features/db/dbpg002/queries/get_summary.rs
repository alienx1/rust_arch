//! DBPG002 query: KPI summary of branches and departments

use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{Dbpg002SummaryDto, db_branch, db_department};
use crate::features::db::dbpg002::queries::db_dbpg002_queries::DbDbpg002Queries;
use crate::features::db::view_scope_filters::{by_branch_data, by_branch_rows, by_company};

impl DbDbpg002Queries {
    pub(super) async fn get_summary(&self) -> Result<Dbpg002SummaryDto, Dbpg002Error> {
        let branches = || {
            by_branch_rows(by_company(db_branch::Entity::find(), db_branch::Column::CompanyId), db_branch::Column::Id)
        };
        let departments = || {
            by_branch_data(
                by_company(db_department::Entity::find(), db_department::Column::CompanyId),
                db_department::Column::BranchId,
                db_department::Column::Id,
            )
        };
        Ok(Dbpg002SummaryDto {
            total_branches: branches().count(&self.db).await?,
            active_branches: branches().filter(db_branch::Column::Active.eq(true)).count(&self.db).await?,
            total_departments: departments().count(&self.db).await?,
            active_departments: departments().filter(db_department::Column::Active.eq(true)).count(&self.db).await?,
        })
    }
}
