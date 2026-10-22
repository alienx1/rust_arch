//! DBPG002 query: active permission groups for a department's default group

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::PermissionGroupOptionDto;
use crate::features::db::dbpg002::queries::db_dbpg002_queries::DbDbpg002Queries;
use domain::entity::su_permission_group;

impl DbDbpg002Queries {
    pub(super) async fn list_permission_group_options(&self) -> Result<Vec<PermissionGroupOptionDto>, Dbpg002Error> {
        Ok(su_permission_group::Entity::find()
            .filter(su_permission_group::Column::Active.eq(true))
            .order_by_asc(su_permission_group::Column::Code)
            .all(&self.db)
            .await?
            .into_iter()
            .map(|g| PermissionGroupOptionDto { id: g.id, code: g.code })
            .collect())
    }
}
