//! DBPG000 query: list all organizations with localized names and company count

use sea_orm::{EntityTrait, QueryOrder};
use std::collections::HashMap;
use uuid::Uuid;

use crate::features::db::dbpg000::errors::Dbpg000Error;
use crate::features::db::dbpg000::models::{
    ENGLISH_LANGUAGE_ID, OrganizationDto, THAI_LANGUAGE_ID, db_company, db_organization,
    db_organization_label,
};

use super::db_dbpg000_queries::DbDbpg000Queries;

impl DbDbpg000Queries {
    pub(super) async fn list_organizations(&self) -> Result<Vec<OrganizationDto>, Dbpg000Error> {
        let orgs = db_organization::Entity::find()
            .order_by_asc(db_organization::Column::Code)
            .all(&self.db)
            .await?;

        let labels = db_organization_label::Entity::find().all(&self.db).await?;
        let mut label_map: HashMap<(Uuid, Uuid), String> = HashMap::new();
        for l in labels {
            label_map.insert((l.organization_id, l.language_id), l.label);
        }

        let companies = db_company::Entity::find().all(&self.db).await?;
        let mut company_counts: HashMap<Uuid, u64> = HashMap::new();
        for c in companies {
            *company_counts.entry(c.organization_id).or_insert(0) += 1;
        }

        let results = orgs
            .into_iter()
            .map(|org| {
                let name_th = label_map
                    .get(&(org.id, THAI_LANGUAGE_ID))
                    .cloned()
                    .unwrap_or_default();
                let name_en = label_map
                    .get(&(org.id, ENGLISH_LANGUAGE_ID))
                    .cloned()
                    .unwrap_or_default();
                let count = company_counts.get(&org.id).copied().unwrap_or(0);

                OrganizationDto {
                    id: org.id,
                    code: org.code,
                    registration_number: org.registration_number,
                    active: org.active,
                    name_th,
                    name_en,
                    company_count: count,
                    row_version: org.row_version,
                }
            })
            .collect();

        Ok(results)
    }
}
