//! DBPG001 command: create a new organization with Thai/English labels

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::info;
use uuid::Uuid;

use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::{
    CreateOrganizationCommand, ENGLISH_LANGUAGE_ID, OrganizationDto, THAI_LANGUAGE_ID,
    db_organization, db_organization_label,
};

use super::dbpg001_executor::Dbpg001CommandExecutor;

impl Dbpg001CommandExecutor {
    pub(super) async fn create_organization(
        &self,
        cmd: CreateOrganizationCommand,
        actor_id: Option<Uuid>,
    ) -> Result<OrganizationDto, Dbpg001Error> {
        let code = cmd.code.trim().to_uppercase();
        if code.is_empty() {
            return Err(Dbpg001Error::Validation(
                "Organization code is required".to_string(),
            ));
        }
        let name_th = cmd.name_th.trim().to_string();
        if name_th.is_empty() {
            return Err(Dbpg001Error::Validation(
                "Thai name is required".to_string(),
            ));
        }
        let name_en = if cmd.name_en.trim().is_empty() {
            name_th.clone()
        } else {
            cmd.name_en.trim().to_string()
        };

        // Check duplicate code
        let existing = db_organization::Entity::find()
            .filter(db_organization::Column::Code.eq(&code))
            .one(&self.db)
            .await?;
        if existing.is_some() {
            return Err(Dbpg001Error::DuplicateCode(format!(
                "Organization code '{code}' already exists"
            )));
        }

        let new_id = Uuid::now_v7();
        info!("Creating new organization: id={new_id}, code={code}");

        // Insert organization
        let org_model = db_organization::ActiveModel {
            id: Set(new_id),
            code: Set(code),
            registration_number: Set(cmd.registration_number.trim().to_string()),
            active: Set(cmd.active),
            change_by: Set(actor_id),
            row_version: Set(1),
        };
        org_model.insert(&self.db).await?;

        // Insert Thai label
        let label_th = db_organization_label::ActiveModel {
            organization_id: Set(new_id),
            language_id: Set(THAI_LANGUAGE_ID),
            label: Set(name_th),
            change_by: Set(actor_id),
            row_version: Set(1),
        };
        label_th.insert(&self.db).await?;

        // Insert English label
        let label_en = db_organization_label::ActiveModel {
            organization_id: Set(new_id),
            language_id: Set(ENGLISH_LANGUAGE_ID),
            label: Set(name_en),
            change_by: Set(actor_id),
            row_version: Set(1),
        };
        label_en.insert(&self.db).await?;

        self.queries
            .get_organization_by_id(new_id)
            .await?
            .ok_or_else(|| {
                Dbpg001Error::Internal("Failed to retrieve created organization".to_string())
            })
    }
}
