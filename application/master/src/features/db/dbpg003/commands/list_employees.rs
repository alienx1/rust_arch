//! DBPG003 read: one page of employees matching the filter.

use crate::features::db::dbpg003::commands::dbpg003_executor::Dbpg003CommandExecutor;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::{EmployeeFilterQuery, EmployeePageDto};

impl Dbpg003CommandExecutor {
    pub(super) async fn list_employees(
        &self,
        filter: EmployeeFilterQuery,
    ) -> Result<EmployeePageDto, Dbpg003Error> {
        self.queries.list_employees(filter).await
    }
}
