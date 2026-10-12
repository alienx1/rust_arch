//! DBPG000 query: retrieve a single company by ID

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::features::db::dbpg000::errors::Dbpg000Error;
use crate::features::db::dbpg000::models::{
    CompanyDto, ENGLISH_LANGUAGE_ID, THAI_LANGUAGE_ID, db_company, db_company_label,
    db_organization, db_organization_label,
};
use crate::features::localization::request_language::pick;
use domain::entity::db_master_lookup_label;

use super::db_dbpg000_queries::DbDbpg000Queries;

impl DbDbpg000Queries {
    pub(super) async fn get_company_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<CompanyDto>, Dbpg000Error> {
        let comp = match db_company::Entity::find_by_id(id).one(&self.db).await? {
            Some(c) => c,
            None => return Ok(None),
        };

        let org = db_organization::Entity::find_by_id(comp.organization_id)
            .one(&self.db)
            .await?;
        let org_code = org.as_ref().map(|o| o.code.clone()).unwrap_or_default();

        let org_labels = db_organization_label::Entity::find()
            .filter(db_organization_label::Column::OrganizationId.eq(comp.organization_id))
            .all(&self.db)
            .await?;
        let org_text = |language_id: Uuid| {
            org_labels
                .iter()
                .find(|l| l.language_id == language_id)
                .map(|l| l.label.clone())
                .unwrap_or_default()
        };
        let org_name = pick(&org_text(THAI_LANGUAGE_ID), &org_text(ENGLISH_LANGUAGE_ID)).to_string();

        let labels = db_company_label::Entity::find()
            .filter(db_company_label::Column::CompanyId.eq(id))
            .all(&self.db)
            .await?;

        let mut name_th = String::new();
        let mut name_en = String::new();
        for l in labels {
            if l.language_id == THAI_LANGUAGE_ID {
                name_th = l.label;
            } else if l.language_id == ENGLISH_LANGUAGE_ID {
                name_en = l.label;
            }
        }

        let type_labels = db_master_lookup_label::Entity::find()
            .filter(db_master_lookup_label::Column::MasterLookupId.eq(comp.master_lookup_id))
            .all(&self.db)
            .await?;
        let type_text = |language_id: Uuid| {
            type_labels
                .iter()
                .find(|l| l.language_id == language_id)
                .map(|l| l.label.clone())
                .unwrap_or_default()
        };
        let company_type_name = match pick(&type_text(THAI_LANGUAGE_ID), &type_text(ENGLISH_LANGUAGE_ID)) {
            "" => "บริษัทจำกัด".to_string(),
            name => name.to_string(),
        };

        Ok(Some(CompanyDto {
            id: comp.id,
            organization_id: comp.organization_id,
            organization_code: org_code,
            organization_name: org_name,
            code: comp.code,
            master_lookup_id: comp.master_lookup_id,
            company_type_name,
            active: comp.active,
            name_th,
            name_en,
            row_version: comp.row_version,
        }))
    }
}
