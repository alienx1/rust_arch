//! DBPG003 read: all departments (filtered client-side by branch).

use crate::features::db::dbpg003::commands::dbpg003_executor::Dbpg003CommandExecutor;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::DepartmentOptionDto;

impl Dbpg003CommandExecutor {
    pub(super) async fn list_department_options(
        &self,
    ) -> Result<Vec<DepartmentOptionDto>, Dbpg003Error> {
        self.queries.list_department_options().await
    }
}
