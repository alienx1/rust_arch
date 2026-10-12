//! DBPG003 reference checks: department exists, prefix exists, employee code unique per department

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::features::db::dbpg002::models::db_department;
use crate::features::db::dbpg003::commands::dbpg003_executor::Dbpg003CommandExecutor;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::db_employee;
use domain::entity::db_master_lookup;

impl Dbpg003CommandExecutor {
    pub(super) async fn department(&self, id: Uuid) -> Result<db_department::Model, Dbpg003Error> {
        db_department::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg003Error::NotFound(format!("Department '{id}' does not exist")))
    }

    pub(super) async fn check_prefix(&self, prefix_id: Uuid) -> Result<(), Dbpg003Error> {
        let found = db_master_lookup::Entity::find_by_id(prefix_id)
            .filter(db_master_lookup::Column::Category.eq("prefix"))
            .one(&self.db)
            .await?;
        if found.is_none() {
            return Err(Dbpg003Error::Validation(format!(
                "Prefix '{prefix_id}' does not exist"
            )));
        }
        Ok(())
    }

    pub(super) async fn check_duplicate(
        &self,
        department: &db_department::Model,
        code: &str,
        exclude: Option<Uuid>,
    ) -> Result<(), Dbpg003Error> {
        let mut select = db_employee::Entity::find()
            .filter(db_employee::Column::OrganizationId.eq(department.organization_id))
            .filter(db_employee::Column::CompanyId.eq(department.company_id))
            .filter(db_employee::Column::BranchId.eq(department.branch_id))
            .filter(db_employee::Column::DepartmentId.eq(department.id))
            .filter(db_employee::Column::Code.eq(code));
        if let Some(id) = exclude {
            select = select.filter(db_employee::Column::Id.ne(id));
        }
        if select.one(&self.db).await?.is_some() {
            return Err(Dbpg003Error::DuplicateCode(format!(
                "Employee code '{code}' already exists in this department"
            )));
        }
        Ok(())
    }
}
