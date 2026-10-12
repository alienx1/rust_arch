//! SUPG005 helper: the scope level a user's primary assignment sits at.

use domain::entity::sea_orm_active_enums::ScopeLevel;
use domain::entity::su_user_assignment;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::features::su::supg005::errors::Supg005Error;
use crate::features::su::supg005::queries::DbSupg005Queries;

impl DbSupg005Queries {
    /// `organization` / `company` / `branch` / `department` of the user's first assignment, or
    /// `any` for a user without assignment rows (the primary group then applies anywhere)
    pub(super) async fn primary_scope_level(&self, user_id: Uuid) -> Result<String, Supg005Error> {
        let first = su_user_assignment::Entity::find()
            .filter(su_user_assignment::Column::UserId.eq(user_id))
            .order_by_asc(su_user_assignment::Column::Id)
            .one(&self.db)
            .await?;
        Ok(match first.map(|a| a.scope_level) {
            Some(ScopeLevel::Organization) => "organization",
            Some(ScopeLevel::Company) => "company",
            Some(ScopeLevel::Branch) => "branch",
            Some(ScopeLevel::Department) => "department",
            None => "any",
        }
        .to_string())
    }
}
