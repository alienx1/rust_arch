//! SUPG001 list modules (system / active filters + search term, ordered by system then code)

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use super::db_supg001_queries::DbSupg001Queries;
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{ModuleDto, ModuleFilterQuery, db_module};

impl DbSupg001Queries {
    pub(super) async fn list_modules(&self, filter: ModuleFilterQuery) -> Result<Vec<ModuleDto>, Supg001Error> {
        let mut select = db_module::Entity::find();
        if let Some(s) = filter.system_id {
            select = select.filter(db_module::Column::SystemId.eq(s));
        }
        if let Some(a) = filter.active {
            select = select.filter(db_module::Column::Active.eq(a));
        }
        let rows = select
            .order_by_asc(db_module::Column::SystemId)
            .order_by_asc(db_module::Column::Code)
            .all(&self.db)
            .await?;
        let term = filter.search.as_deref().unwrap_or("").trim().to_lowercase();
        self.module_dtos(rows, &term).await
    }
}
