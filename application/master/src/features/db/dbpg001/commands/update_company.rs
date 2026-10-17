//! DBPG001 command: update a company with optimistic lock verification

use sea_orm::{TransactionTrait, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::features::optimistic_update::update_versioned;

use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::{
    CompanyDto, ENGLISH_LANGUAGE_ID, THAI_LANGUAGE_ID, UpdateCompanyCommand, db_company,
    db_company_label,
};

use super::dbpg001_executor::Dbpg001CommandExecutor;

impl Dbpg001CommandExecutor {
    pub(super) async fn update_company(
        &self,
        id: Uuid,
        cmd: UpdateCompanyCommand,
        actor_id: Option<Uuid>,
    ) -> Result<CompanyDto, Dbpg001Error> {
        let code = cmd.code.trim().to_uppercase();
        if code.is_empty() {
            return Err(Dbpg001Error::Validation(
                "Company code is required".to_string(),
            ));
        }

        let existing = db_company::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg001Error::NotFound(id.to_string()))?;


        // Check duplicate code if changed
        if existing.code != code || existing.organization_id != cmd.organization_id {
            let dup = db_company::Entity::find()
                .filter(db_company::Column::OrganizationId.eq(cmd.organization_id))
                .filter(db_company::Column::Code.eq(&code))
                .filter(db_company::Column::Id.ne(id))
                .one(&self.db)
                .await?;
            if dup.is_some() {
                return Err(Dbpg001Error::DuplicateCode(format!(
                    "Company code '{code}' already exists under this organization"
                )));
            }
        }

        let next_version = existing.row_version + 1;
        let txn = self.db.begin().await?;
        let mut active_model: db_company::ActiveModel = existing.into();
        active_model.organization_id = Set(cmd.organization_id);
        active_model.code = Set(code);
        active_model.master_lookup_id = Set(cmd.master_lookup_id);
        active_model.active = Set(cmd.active);
        active_model.change_by = Set(actor_id);
        update_versioned(&txn, active_model, (db_company::Column::Id, db_company::Column::RowVersion), id, cmd.row_version).await?;

        // Update Thai label
        let name_th = cmd.name_th.trim().to_string();
        if !name_th.is_empty() {
            match db_company_label::Entity::find_by_id((id, THAI_LANGUAGE_ID))
                .one(&txn)
                .await?
            {
                Some(existing_lbl) => {
                    let mut lbl_am: db_company_label::ActiveModel = existing_lbl.into();
                    lbl_am.label = Set(name_th);
                    lbl_am.change_by = Set(actor_id);
                    lbl_am.row_version = Set(next_version);
                    lbl_am.update(&txn).await?;
                }
                None => {
                    let lbl_am = db_company_label::ActiveModel {
                        company_id: Set(id),
                        language_id: Set(THAI_LANGUAGE_ID),
                        label: Set(name_th),
                        change_by: Set(actor_id),
                        row_version: Set(1),
                    };
                    lbl_am.insert(&txn).await?;
                }
            }
        }

        // Update English label
        let name_en = cmd.name_en.trim().to_string();
        if !name_en.is_empty() {
            match db_company_label::Entity::find_by_id((id, ENGLISH_LANGUAGE_ID))
                .one(&txn)
                .await?
            {
                Some(existing_lbl) => {
                    let mut lbl_am: db_company_label::ActiveModel = existing_lbl.into();
                    lbl_am.label = Set(name_en);
                    lbl_am.change_by = Set(actor_id);
                    lbl_am.row_version = Set(next_version);
                    lbl_am.update(&txn).await?;
                }
                None => {
                    let lbl_am = db_company_label::ActiveModel {
                        company_id: Set(id),
                        language_id: Set(ENGLISH_LANGUAGE_ID),
                        label: Set(name_en),
                        change_by: Set(actor_id),
                        row_version: Set(1),
                    };
                    lbl_am.insert(&txn).await?;
                }
            }
        }

        txn.commit().await?;

        self.queries
            .get_company_by_id(id)
            .await?
            .ok_or_else(|| Dbpg001Error::Internal("Failed to retrieve updated company".to_string()))
    }
}
