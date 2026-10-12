//! SUPG001 list systems (active flag + search term, ordered by code)

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use super::db_supg001_queries::DbSupg001Queries;
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{SystemDto, SystemFilterQuery, db_system};

impl DbSupg001Queries {
    pub(super) async fn list_systems(&self, filter: SystemFilterQuery) -> Result<Vec<SystemDto>, Supg001Error> {
        let mut select = db_system::Entity::find();
        if let Some(a) = filter.active {
            select = select.filter(db_system::Column::Active.eq(a));
        }
        let rows = select.order_by_asc(db_system::Column::Code).all(&self.db).await?;
        let term = filter.search.as_deref().unwrap_or("").trim().to_lowercase();
        self.system_dtos(rows, &term).await
    }
}
