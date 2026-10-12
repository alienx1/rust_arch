//! DBPG002 read: departments with optional filters.

use crate::features::db::dbpg002::commands::dbpg002_executor::Dbpg002CommandExecutor;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{DepartmentDto, DepartmentFilterQuery};

impl Dbpg002CommandExecutor {
    pub(super) async fn list_departments(
        &self,
        filter: DepartmentFilterQuery,
    ) -> Result<Vec<DepartmentDto>, Dbpg002Error> {
        self.queries.list_departments(filter).await
    }
}
