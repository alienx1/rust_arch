//! DBPG002 write: update a branch with optimistic lock verification

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use uuid::Uuid;

use crate::features::optimistic_update::update_versioned;

use super::branch_label_writer::upsert_branch_label;
use super::name_rules::required;
use crate::features::db::dbpg002::commands::dbpg002_executor::Dbpg002CommandExecutor;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{
    BranchDto, ENGLISH_LANGUAGE_ID, THAI_LANGUAGE_ID, UpdateBranchCommand, db_branch,
};

impl Dbpg002CommandExecutor {
    pub(super) async fn update_branch(
        &self,
        id: Uuid,
        cmd: UpdateBranchCommand,
        actor_id: Option<Uuid>,
    ) -> Result<BranchDto, Dbpg002Error> {
        let code = required(&cmd.code, "Branch code")?.to_uppercase();

        let existing = db_branch::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg002Error::NotFound(id.to_string()))?;

        if existing.code != code {
            let duplicate = db_branch::Entity::find()
                .filter(db_branch::Column::OrganizationId.eq(existing.organization_id))
                .filter(db_branch::Column::CompanyId.eq(existing.company_id))
                .filter(db_branch::Column::Code.eq(&code))
                .filter(db_branch::Column::Id.ne(id))
                .one(&self.db)
                .await?;
            if duplicate.is_some() {
                return Err(Dbpg002Error::DuplicateCode(format!(
                    "Branch code '{code}' already exists under this company"
                )));
            }
        }
        self.check_branch_type(existing.company_id, cmd.master_lookup_id, Some(id))
            .await?;

        let next_version = existing.row_version + 1;
        let txn = self.db.begin().await?;
        let mut am: db_branch::ActiveModel = existing.into();
        am.code = Set(code);
        am.master_lookup_id = Set(cmd.master_lookup_id);
        am.active = Set(cmd.active);
        am.change_by = Set(actor_id);
        update_versioned(&txn, am, (db_branch::Column::Id, db_branch::Column::RowVersion), id, cmd.row_version).await?;

        let name_th = cmd.name_th.trim().to_string();
        if !name_th.is_empty() {
            upsert_branch_label(&txn, id, THAI_LANGUAGE_ID, name_th, actor_id, next_version)
                .await?;
        }
        let name_en = cmd.name_en.trim().to_string();
        if !name_en.is_empty() {
            upsert_branch_label(
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

        self.queries
            .get_branch_by_id(id)
            .await?
            .ok_or_else(|| Dbpg002Error::Internal("Failed to retrieve updated branch".to_string()))
    }
}
