//! Persist the account active flag

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use domain::entity::su_user;

use crate::features::su::supg005::errors::Supg005Error;
use crate::features::su::supg005::queries::DbSupg005Queries;

impl DbSupg005Queries {
    pub(super) async fn set_user_active(&self, user_id: &str, active: bool) -> Result<bool, Supg005Error> {
        use sea_orm::sea_query::{Expr, ExprTrait};
        let Ok(id) = user_id.parse::<uuid::Uuid>() else {
            return Ok(false);
        };
        let res = su_user::Entity::update_many()
            .col_expr(su_user::Column::Active, Expr::value(active))
            .col_expr(su_user::Column::RowVersion, Expr::col(su_user::Column::RowVersion).add(1))
            .filter(su_user::Column::Id.eq(id))
            .exec(&self.db)
            .await?;
        Ok(res.rows_affected > 0)
    }
}
