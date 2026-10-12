//! DBPG000 query: retrieve a single organization by ID

use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use uuid::Uuid;

use crate::features::db::dbpg000::errors::Dbpg000Error;
use crate::features::db::dbpg000::models::{
    ENGLISH_LANGUAGE_ID, OrganizationDto, THAI_LANGUAGE_ID, db_company, db_organization,
    db_organization_label,
};

use super::db_dbpg000_queries::DbDbpg000Queries;

impl DbDbpg000Queries {
    pub(super) async fn get_organization_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<OrganizationDto>, Dbpg000Error> {
        let org = match db_organization::Entity::find_by_id(id)
            .one(&self.db)
            .await?
        {
            Some(o) => o,
            None => return Ok(None),
        };

        let labels = db_organization_label::Entity::find()
            .filter(db_organization_label::Column::OrganizationId.eq(id))
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

        let count = db_company::Entity::find()
            .filter(db_company::Column::OrganizationId.eq(id))
            .count(&self.db)
            .await?;

        Ok(Some(OrganizationDto {
            id: org.id,
            code: org.code,
            registration_number: org.registration_number,
            active: org.active,
            name_th,
            name_en,
            company_count: count,
            row_version: org.row_version,
        }))
    }
}
