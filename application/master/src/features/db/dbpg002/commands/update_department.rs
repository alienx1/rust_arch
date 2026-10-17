//! DBPG002 write: update a department (optionally moving it to another branch) with optimistic lock

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use uuid::Uuid;

use crate::features::optimistic_update::update_versioned;

use super::department_label_writer::upsert_department_label;
use super::name_rules::required;
use crate::features::db::dbpg002::commands::dbpg002_executor::Dbpg002CommandExecutor;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{
    DepartmentDto, ENGLISH_LANGUAGE_ID, THAI_LANGUAGE_ID, UpdateDepartmentCommand, db_branch,
    db_department,
};

impl Dbpg002CommandExecutor {
    pub(super) async fn update_department(
        &self,
        id: Uuid,
        cmd: UpdateDepartmentCommand,
        actor_id: Option<Uuid>,
    ) -> Result<DepartmentDto, Dbpg002Error> {
        let code = required(&cmd.code, "Department code")?.to_uppercase();

        let existing = db_department::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg002Error::NotFound(id.to_string()))?;

        let branch = db_branch::Entity::find_by_id(cmd.branch_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| {
                Dbpg002Error::NotFound(format!("Branch '{}' does not exist", cmd.branch_id))
            })?;

        if existing.code != code || existing.branch_id != branch.id {
            let duplicate = db_department::Entity::find()
                .filter(db_department::Column::OrganizationId.eq(branch.organization_id))
                .filter(db_department::Column::CompanyId.eq(branch.company_id))
                .filter(db_department::Column::BranchId.eq(branch.id))
                .filter(db_department::Column::Code.eq(&code))
                .filter(db_department::Column::Id.ne(id))
                .one(&self.db)
                .await?;
            if duplicate.is_some() {
                return Err(Dbpg002Error::DuplicateCode(format!(
                    "Department code '{code}' already exists under this branch"
                )));
            }
        }

        let next_version = existing.row_version + 1;
        let txn = self.db.begin().await?;
        let mut am: db_department::ActiveModel = existing.into();
        am.organization_id = Set(branch.organization_id);
        am.company_id = Set(branch.company_id);
        am.branch_id = Set(branch.id);
        am.code = Set(code);
        am.active = Set(cmd.active);
        am.change_by = Set(actor_id);
        update_versioned(&txn, am, (db_department::Column::Id, db_department::Column::RowVersion), id, cmd.row_version).await?;

        let name_th = cmd.name_th.trim().to_string();
        if !name_th.is_empty() {
            upsert_department_label(&txn, id, THAI_LANGUAGE_ID, name_th, actor_id, next_version)
                .await?;
        }
        let name_en = cmd.name_en.trim().to_string();
        if !name_en.is_empty() {
            upsert_department_label(
                &txn,
                id,
                ENGLISH_LANGUAGE_ID,
                name_en,
                actor_id,
                next_version,
            )
            .await?;
        }
        txn.commit().await?;

        self.queries.get_department_by_id(id).await?.ok_or_else(|| {
            Dbpg002Error::Internal("Failed to retrieve updated department".to_string())
        })
    }
}
