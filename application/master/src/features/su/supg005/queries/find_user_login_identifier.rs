//! Resolve the identity-provider login identifier of a user

use domain::entity::su_user;
use sea_orm::EntityTrait;

use crate::features::su::supg005::errors::Supg005Error;
use crate::features::su::supg005::queries::DbSupg005Queries;

impl DbSupg005Queries {
    pub(super) async fn find_user_login_identifier(&self, user_id: &str) -> Result<Option<String>, Supg005Error> {
        let Ok(id) = user_id.parse::<uuid::Uuid>() else {
            return Ok(None);
        };
        let user = su_user::Entity::find_by_id(id).one(&self.db).await?;
        // The username is a credential identifier at the identity provider, whatever the email is
        Ok(user.map(|u| u.username))
    }
}
