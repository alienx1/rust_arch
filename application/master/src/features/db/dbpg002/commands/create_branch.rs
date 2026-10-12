//! DBPG002 write: create a branch with its address/contact groups and labels (one transaction)

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tracing::info;
use uuid::Uuid;

use super::branch_label_writer::upsert_branch_label;
use super::name_rules::{names_for_create, required};
use crate::features::db::dbpg002::commands::dbpg002_executor::Dbpg002CommandExecutor;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{
    BranchDto, CreateBranchCommand, ENGLISH_LANGUAGE_ID, THAI_LANGUAGE_ID, db_address_group,
    db_branch, db_company, db_contact_group,
};

impl Dbpg002CommandExecutor {
    pub(super) async fn create_branch(
        &self,
        cmd: CreateBranchCommand,
        actor_id: Option<Uuid>,
    ) -> Result<BranchDto, Dbpg002Error> {
        let code = required(&cmd.code, "Branch code")?.to_uppercase();
        let (name_th, name_en) = names_for_create(&cmd.name_th, &cmd.name_en)?;

        let company = db_company::Entity::find_by_id(cmd.company_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| {
                Dbpg002Error::NotFound(format!("Company '{}' does not exist", cmd.company_id))
            })?;

        let duplicate = db_branch::Entity::find()
            .filter(db_branch::Column::OrganizationId.eq(company.organization_id))
            .filter(db_branch::Column::CompanyId.eq(company.id))
            .filter(db_branch::Column::Code.eq(&code))
            .one(&self.db)
            .await?;
        if duplicate.is_some() {
            return Err(Dbpg002Error::DuplicateCode(format!(
                "Branch code '{code}' already exists under this company"
            )));
        }
        self.check_branch_type(company.id, cmd.master_lookup_id, None)
            .await?;

        let id = Uuid::now_v7();
        info!("Creating new branch: id={id}, code={code}");

        let txn = self.db.begin().await?;
        let address_group_id = Uuid::now_v7();
        db_address_group::ActiveModel {
            id: Set(address_group_id),
            active: Set(Some(true)),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
        .insert(&txn)
        .await?;
        let contact_group_id = Uuid::now_v7();
        db_contact_group::ActiveModel {
            id: Set(contact_group_id),
            active: Set(Some(true)),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
        .insert(&txn)
        .await?;
        db_branch::ActiveModel {
            id: Set(id),
            organization_id: Set(company.organization_id),
            company_id: Set(company.id),
            code: Set(code),
            master_lookup_id: Set(cmd.master_lookup_id),
            address_group_id: Set(address_group_id),
            contact_group_id: Set(contact_group_id),
            active: Set(cmd.active),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
        .insert(&txn)
        .await?;
        upsert_branch_label(&txn, id, THAI_LANGUAGE_ID, name_th, actor_id, 1).await?;
        upsert_branch_label(&txn, id, ENGLISH_LANGUAGE_ID, name_en, actor_id, 1).await?;
        txn.commit().await?;

        self.queries
            .get_branch_by_id(id)
            .await?
            .ok_or_else(|| Dbpg002Error::Internal("Failed to retrieve created branch".to_string()))
    }
}
