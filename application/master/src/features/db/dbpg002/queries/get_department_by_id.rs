//! DBPG002 query: a single department by id

use sea_orm::EntityTrait;
use uuid::Uuid;

use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{DepartmentDto, db_department};
use crate::features::db::dbpg002::queries::db_dbpg002_queries::DbDbpg002Queries;
use crate::features::db::view_scope_filters::in_scope;

impl DbDbpg002Queries {
    pub(super) async fn get_department_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<DepartmentDto>, Dbpg002Error> {
        match db_department::Entity::find_by_id(id).one(&self.db).await? {
            Some(dept) if in_scope(Some(dept.company_id), Some(dept.branch_id), Some(dept.id)) => Ok(self
                .department_dtos(vec![dept], None)
                .await?
                .into_iter()
                .next()),
            _ => Ok(None),
        }
    }
}
