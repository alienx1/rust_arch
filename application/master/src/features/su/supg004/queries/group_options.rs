//! SUPG004 group-options query: every permission group an account can be placed in.

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use super::db_supg004_queries::DbSupg004Queries;
use crate::features::su::supg004::errors::Supg004Error;
use crate::features::su::supg004::models::{GroupOptionDto, is_admin_group};
use domain::entity::su_permission_group;

impl DbSupg004Queries {
    pub(super) async fn group_options(&self) -> Result<Vec<GroupOptionDto>, Supg004Error> {
        let groups = su_permission_group::Entity::find()
            .filter(su_permission_group::Column::Active.eq(true))
            .order_by_asc(su_permission_group::Column::Code)
            .all(&self.db)
            .await?;
        Ok(groups
            .into_iter()
            .map(|g| GroupOptionDto {
                id: g.id,
                needs_approval: is_admin_group(&g.code),
                code: g.code,
                active: g.active,
            })
            .collect())
    }
}
