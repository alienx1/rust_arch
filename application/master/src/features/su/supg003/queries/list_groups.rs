//! SUPG003 list-groups query: groups filtered by status and search term, ordered by code.

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use super::db_supg003_queries::DbSupg003Queries;
use crate::features::su::supg003::errors::Supg003Error;
use crate::features::su::supg003::models::{GroupFilterQuery, PermissionGroupDto, su_permission_group};

impl DbSupg003Queries {
    pub(super) async fn list_groups(&self, filter: GroupFilterQuery) -> Result<Vec<PermissionGroupDto>, Supg003Error> {
        let mut select = su_permission_group::Entity::find();
        if let Some(a) = filter.active {
            select = select.filter(su_permission_group::Column::Active.eq(a));
        }
        let rows = select.order_by_asc(su_permission_group::Column::Code).all(&self.db).await?;
        let term = filter.search.as_deref().unwrap_or("").trim().to_lowercase();
        self.group_dtos(rows, &term).await
    }
}
