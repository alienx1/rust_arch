//! DBPG002 query: a single branch by id

use sea_orm::EntityTrait;
use uuid::Uuid;

use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{BranchDto, db_branch};
use crate::features::db::dbpg002::queries::db_dbpg002_queries::DbDbpg002Queries;
use crate::features::db::view_scope_filters::branch_in_scope;

impl DbDbpg002Queries {
    pub(super) async fn get_branch_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<BranchDto>, Dbpg002Error> {
        match db_branch::Entity::find_by_id(id).one(&self.db).await? {
            Some(branch) if branch_in_scope(branch.company_id, branch.id) => Ok(self
                .branch_dtos(vec![branch], None)
                .await?
                .into_iter()
                .next()),
            _ => Ok(None),
        }
    }
}
