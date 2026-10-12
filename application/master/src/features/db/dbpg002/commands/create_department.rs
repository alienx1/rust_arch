//! DBPG002 write: create a department under a branch with its labels (one transaction)

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tracing::info;
use uuid::Uuid;

use super::department_label_writer::upsert_department_label;
use super::name_rules::{names_for_create, required};
use crate::features::db::dbpg002::commands::dbpg002_executor::Dbpg002CommandExecutor;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{
    CreateDepartmentCommand, DepartmentDto, ENGLISH_LANGUAGE_ID, THAI_LANGUAGE_ID, db_branch,
    db_department,
};

impl Dbpg002CommandExecutor {
    pub(super) async fn create_department(
        &self,
        cmd: CreateDepartmentCommand,
        actor_id: Option<Uuid>,
    ) -> Result<DepartmentDto, Dbpg002Error> {
        let code = required(&cmd.code, "Department code")?.to_uppercase();
        let (name_th, name_en) = names_for_create(&cmd.name_th, &cmd.name_en)?;

        let branch = db_branch::Entity::find_by_id(cmd.branch_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| {
                Dbpg002Error::NotFound(format!("Branch '{}' does not exist", cmd.branch_id))
            })?;

        let duplicate = db_department::Entity::find()
            .filter(db_department::Column::OrganizationId.eq(branch.organization_id))
            .filter(db_department::Column::CompanyId.eq(branch.company_id))
            .filter(db_department::Column::BranchId.eq(branch.id))
            .filter(db_department::Column::Code.eq(&code))
            .one(&self.db)
            .await?;
        if duplicate.is_some() {
            return Err(Dbpg002Error::DuplicateCode(format!(
                "Department code '{code}' already exists under this branch"
            )));
        }

        let id = Uuid::now_v7();
        info!("Creating new department: id={id}, code={code}");

        let txn = self.db.begin().await?;
        db_department::ActiveModel {
            id: Set(id),
            organization_id: Set(branch.organization_id),
            company_id: Set(branch.company_id),
            branch_id: Set(branch.id),
            code: Set(code),
            active: Set(cmd.active),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
        .insert(&txn)
        .await?;
        upsert_department_label(&txn, id, THAI_LANGUAGE_ID, name_th, actor_id, 1).await?;
        upsert_department_label(&txn, id, ENGLISH_LANGUAGE_ID, name_en, actor_id, 1).await?;
        txn.commit().await?;

        self.queries.get_department_by_id(id).await?.ok_or_else(|| {
            Dbpg002Error::Internal("Failed to retrieve created department".to_string())
        })
    }
}
