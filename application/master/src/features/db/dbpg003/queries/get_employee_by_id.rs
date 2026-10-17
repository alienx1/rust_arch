//! DBPG003 query: a single employee by id

use sea_orm::EntityTrait;
use uuid::Uuid;

use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::{EmployeeDto, db_employee};
use crate::features::db::dbpg003::queries::db_dbpg003_queries::DbDbpg003Queries;
use crate::features::db::view_scope_filters::in_scope;

impl DbDbpg003Queries {
    pub(super) async fn get_employee_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<EmployeeDto>, Dbpg003Error> {
        match db_employee::Entity::find_by_id(id).one(&self.db).await? {
            Some(e) if in_scope(Some(e.company_id), Some(e.branch_id), Some(e.department_id)) => {
                Ok(self.employee_dtos(vec![e]).await?.into_iter().next())
            }
            _ => Ok(None),
        }
    }
}
