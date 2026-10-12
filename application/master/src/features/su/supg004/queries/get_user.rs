//! SUPG004 get-user query: one account by id.

use sea_orm::EntityTrait;
use uuid::Uuid;

use super::db_supg004_queries::DbSupg004Queries;
use crate::features::su::supg004::errors::Supg004Error;
use crate::features::su::supg004::models::{UserAccountDto, su_user};

impl DbSupg004Queries {
    pub(super) async fn get_user(&self, id: Uuid) -> Result<Option<UserAccountDto>, Supg004Error> {
        match su_user::Entity::find_by_id(id).one(&self.db).await? {
            Some(u) => Ok(self.user_dtos(vec![u]).await?.into_iter().next()),
            None => Ok(None),
        }
    }
}
