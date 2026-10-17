//! DBPG001 command: update an organization with optimistic lock verification

use sea_orm::{TransactionTrait, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::features::optimistic_update::update_versioned;

use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::{
    ENGLISH_LANGUAGE_ID, OrganizationDto, THAI_LANGUAGE_ID, UpdateOrganizationCommand,
    db_organization, db_organization_label,
};

use super::dbpg001_executor::Dbpg001CommandExecutor;

impl Dbpg001CommandExecutor {
    pub(super) async fn update_organization(
        &self,
        id: Uuid,
        cmd: UpdateOrganizationCommand,
        actor_id: Option<Uuid>,
    ) -> Result<OrganizationDto, Dbpg001Error> {
        let code = cmd.code.trim().to_uppercase();
        if code.is_empty() {
            return Err(Dbpg001Error::Validation(
                "Organization code is required".to_string(),
            ));
        }

        let existing = db_organization::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg001Error::NotFound(id.to_string()))?;


        // Check duplicate code if code changed
        if existing.code != code {
            let dup = db_organization::Entity::find()
                .filter(db_organization::Column::Code.eq(&code))
                .filter(db_organization::Column::Id.ne(id))
                .one(&self.db)
                .await?;
            if dup.is_some() {
                return Err(Dbpg001Error::DuplicateCode(format!(
                    "Organization code '{code}' already exists"
                )));
            }
        }

        let next_version = existing.row_version + 1;
        let txn = self.db.begin().await?;
        let mut active_model: db_organization::ActiveModel = existing.into();
        active_model.code = Set(code);
        active_model.registration_number = Set(cmd.registration_number.trim().to_string());
        active_model.active = Set(cmd.active);
        active_model.change_by = Set(actor_id);
        update_versioned(&txn, active_model, (db_organization::Column::Id, db_organization::Column::RowVersion), id, cmd.row_version).await?;

        // Update Thai label
        let name_th = cmd.name_th.trim().to_string();
        if !name_th.is_empty() {
            match db_organization_label::Entity::find_by_id((id, THAI_LANGUAGE_ID))
                .one(&txn)
                .await?
            {
                Some(existing_lbl) => {
                    let mut lbl_am: db_organization_label::ActiveModel = existing_lbl.into();
                    lbl_am.label = Set(name_th);
                    lbl_am.change_by = Set(actor_id);
                    lbl_am.row_version = Set(next_version);
                    lbl_am.update(&txn).await?;
                }
                None => {
                    let lbl_am = db_organization_label::ActiveModel {
                        organization_id: Set(id),
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
            match db_organization_label::Entity::find_by_id((id, ENGLISH_LANGUAGE_ID))
                .one(&txn)
                .await?
            {
                Some(existing_lbl) => {
                    let mut lbl_am: db_organization_label::ActiveModel = existing_lbl.into();
                    lbl_am.label = Set(name_en);
                    lbl_am.change_by = Set(actor_id);
                    lbl_am.row_version = Set(next_version);
                    lbl_am.update(&txn).await?;
                }
                None => {
                    let lbl_am = db_organization_label::ActiveModel {
                        organization_id: Set(id),
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
            .get_organization_by_id(id)
            .await?
            .ok_or_else(|| {
                Dbpg001Error::Internal("Failed to retrieve updated organization".to_string())
            })
    }
}
