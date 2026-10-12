//! SUPG003 get-group query: one group by id.

use sea_orm::EntityTrait;
use uuid::Uuid;

use super::db_supg003_queries::DbSupg003Queries;
use crate::features::su::supg003::errors::Supg003Error;
use crate::features::su::supg003::models::{PermissionGroupDto, su_permission_group};

impl DbSupg003Queries {
    pub(super) async fn get_group(&self, id: Uuid) -> Result<Option<PermissionGroupDto>, Supg003Error> {
        match su_permission_group::Entity::find_by_id(id).one(&self.db).await? {
            Some(g) => Ok(self.group_dtos(vec![g], "").await?.into_iter().next()),
            None => Ok(None),
        }
    }
}
