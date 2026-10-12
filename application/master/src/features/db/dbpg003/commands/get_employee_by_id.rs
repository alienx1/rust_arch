//! DBPG003 read: a single employee by id.

use uuid::Uuid;

use crate::features::db::dbpg003::commands::dbpg003_executor::Dbpg003CommandExecutor;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::EmployeeDto;

impl Dbpg003CommandExecutor {
    pub(super) async fn get_employee_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<EmployeeDto>, Dbpg003Error> {
        self.queries.get_employee_by_id(id).await
    }
}
