//! DBPG001 command: create a new company under an organization

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::info;
use uuid::Uuid;

use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::{
    CompanyDto, CreateCompanyCommand, ENGLISH_LANGUAGE_ID, THAI_LANGUAGE_ID, db_company,
    db_company_label, db_organization,
};

use super::dbpg001_executor::Dbpg001CommandExecutor;

impl Dbpg001CommandExecutor {
    pub(super) async fn create_company(
        &self,
        cmd: CreateCompanyCommand,
        actor_id: Option<Uuid>,
    ) -> Result<CompanyDto, Dbpg001Error> {
        let code = cmd.code.trim().to_uppercase();
        if code.is_empty() {
            return Err(Dbpg001Error::Validation(
                "Company code is required".to_string(),
            ));
        }

        // Ensure parent organization exists
        db_organization::Entity::find_by_id(cmd.organization_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| {
                Dbpg001Error::NotFound(format!(
                    "Parent organization '{}' does not exist",
                    cmd.organization_id
                ))
            })?;

        // Check duplicate code under same organization
        let existing = db_company::Entity::find()
            .filter(db_company::Column::OrganizationId.eq(cmd.organization_id))
            .filter(db_company::Column::Code.eq(&code))
            .one(&self.db)
            .await?;
        if existing.is_some() {
            return Err(Dbpg001Error::DuplicateCode(format!(
                "Company code '{code}' already exists under this organization"
            )));
        }

        let new_id = Uuid::now_v7();
        info!("Creating new company: id={new_id}, code={code}");

        let comp_model = db_company::ActiveModel {
            id: Set(new_id),
            organization_id: Set(cmd.organization_id),
            code: Set(code),
            master_lookup_id: Set(cmd.master_lookup_id),
            active: Set(cmd.active),
            change_by: Set(actor_id),
            row_version: Set(1),
        };
        comp_model.insert(&self.db).await?;

        let name_th = cmd.name_th.trim().to_string();
        let name_en = if cmd.name_en.trim().is_empty() {
            name_th.clone()
        } else {
            cmd.name_en.trim().to_string()
        };

        let label_th = db_company_label::ActiveModel {
            company_id: Set(new_id),
            language_id: Set(THAI_LANGUAGE_ID),
            label: Set(name_th),
            change_by: Set(actor_id),
            row_version: Set(1),
        };
        label_th.insert(&self.db).await?;

        let label_en = db_company_label::ActiveModel {
            company_id: Set(new_id),
            language_id: Set(ENGLISH_LANGUAGE_ID),
            label: Set(name_en),
            change_by: Set(actor_id),
            row_version: Set(1),
        };
        label_en.insert(&self.db).await?;

        self.queries
            .get_company_by_id(new_id)
            .await?
            .ok_or_else(|| Dbpg001Error::Internal("Failed to retrieve created company".to_string()))
    }
}
