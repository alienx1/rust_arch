//! DBPG002 query: KPI summary of branches and departments

use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{Dbpg002SummaryDto, db_branch, db_department};
use crate::features::db::dbpg002::queries::db_dbpg002_queries::DbDbpg002Queries;

impl DbDbpg002Queries {
    pub(super) async fn get_summary(&self) -> Result<Dbpg002SummaryDto, Dbpg002Error> {
        Ok(Dbpg002SummaryDto {
            total_branches: db_branch::Entity::find().count(&self.db).await?,
            active_branches: db_branch::Entity::find()
                .filter(db_branch::Column::Active.eq(true))
                .count(&self.db)
                .await?,
            total_departments: db_department::Entity::find().count(&self.db).await?,
            active_departments: db_department::Entity::find()
                .filter(db_department::Column::Active.eq(true))
                .count(&self.db)
                .await?,
        })
    }
}
