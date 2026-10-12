//! DBPG003 query: KPI summary of employees

use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use std::collections::HashSet;
use uuid::Uuid;

use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::{Dbpg003SummaryDto, db_employee};
use crate::features::db::dbpg003::queries::db_dbpg003_queries::DbDbpg003Queries;
use domain::entity::su_user;

impl DbDbpg003Queries {
    pub(super) async fn get_summary(&self) -> Result<Dbpg003SummaryDto, Dbpg003Error> {
        let linked: HashSet<Uuid> = su_user::Entity::find()
            .all(&self.db)
            .await?
            .into_iter()
            .map(|u| u.employee_id)
            .collect();
        Ok(Dbpg003SummaryDto {
            total_employees: db_employee::Entity::find().count(&self.db).await?,
            active_employees: db_employee::Entity::find()
                .filter(db_employee::Column::Active.eq(true))
                .count(&self.db)
                .await?,
            with_user_account: linked.len() as u64,
        })
    }
}
