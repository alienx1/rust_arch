//! DBPG000 query: list all company types (e.g. PUBLIC, PRIVATE) from db_master_lookup

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use std::collections::HashMap;
use uuid::Uuid;

use crate::features::db::dbpg000::errors::Dbpg000Error;
use crate::features::db::dbpg000::models::{
    CompanyTypeLookupDto, ENGLISH_LANGUAGE_ID, THAI_LANGUAGE_ID,
};
use domain::entity::{db_master_lookup, db_master_lookup_label};

use super::db_dbpg000_queries::DbDbpg000Queries;

impl DbDbpg000Queries {
    pub(super) async fn list_company_types(
        &self,
    ) -> Result<Vec<CompanyTypeLookupDto>, Dbpg000Error> {
        let lookups = db_master_lookup::Entity::find()
            .filter(db_master_lookup::Column::Category.eq("company_type"))
            .filter(db_master_lookup::Column::Active.eq(true))
            .order_by_asc(db_master_lookup::Column::Code)
            .all(&self.db)
            .await?;

        let labels = db_master_lookup_label::Entity::find().all(&self.db).await?;
        let mut label_map: HashMap<(Uuid, Uuid), String> = HashMap::new();
        for l in labels {
            label_map.insert((l.master_lookup_id, l.language_id), l.label);
        }

        let types = lookups
            .into_iter()
            .map(|lookup| {
                let name_th = label_map
                    .get(&(lookup.id, THAI_LANGUAGE_ID))
                    .cloned()
                    .unwrap_or_else(|| lookup.code.clone());
                let name_en = label_map
                    .get(&(lookup.id, ENGLISH_LANGUAGE_ID))
                    .cloned()
                    .unwrap_or_else(|| lookup.code.clone());

                CompanyTypeLookupDto {
                    id: lookup.id,
                    code: lookup.code,
                    name_th,
                    name_en,
                }
            })
            .collect();

        Ok(types)
    }
}
