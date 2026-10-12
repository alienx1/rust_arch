//! DBPG002 query: branches with optional company / search / active filters

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{BranchDto, BranchFilterQuery, db_branch};
use crate::features::db::dbpg002::queries::db_dbpg002_queries::DbDbpg002Queries;

impl DbDbpg002Queries {
    pub(super) async fn list_branches(
        &self,
        filter: BranchFilterQuery,
    ) -> Result<Vec<BranchDto>, Dbpg002Error> {
        let mut select = db_branch::Entity::find();
        if let Some(company_id) = filter.company_id {
            select = select.filter(db_branch::Column::CompanyId.eq(company_id));
        }
        if let Some(active) = filter.active {
            select = select.filter(db_branch::Column::Active.eq(active));
        }
        let branches = select
            .order_by_asc(db_branch::Column::Code)
            .all(&self.db)
            .await?;
        self.branch_dtos(branches, filter.search.as_deref()).await
    }
}
