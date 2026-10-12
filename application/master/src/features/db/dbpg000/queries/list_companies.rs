//! DBPG000 query: list companies with optional filtering by organization, search query, active status

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use std::collections::HashMap;
use uuid::Uuid;

use crate::features::db::dbpg000::errors::Dbpg000Error;
use crate::features::db::dbpg000::models::{
    CompanyDto, CompanyFilterQuery, ENGLISH_LANGUAGE_ID, THAI_LANGUAGE_ID, db_company,
    db_company_label, db_organization, db_organization_label,
};
use crate::features::localization::request_language::pick;
use domain::entity::db_master_lookup_label;

use super::db_dbpg000_queries::DbDbpg000Queries;

impl DbDbpg000Queries {
    pub(super) async fn list_companies(
        &self,
        filter: CompanyFilterQuery,
    ) -> Result<Vec<CompanyDto>, Dbpg000Error> {
        let mut select = db_company::Entity::find();

        if let Some(org_id) = filter.organization_id {
            select = select.filter(db_company::Column::OrganizationId.eq(org_id));
        }

        if let Some(active) = filter.active {
            select = select.filter(db_company::Column::Active.eq(active));
        }

        let companies = select
            .order_by_asc(db_company::Column::Code)
            .all(&self.db)
            .await?;

        // Preload organizations for code & Thai name
        let orgs = db_organization::Entity::find().all(&self.db).await?;
        let org_labels = db_organization_label::Entity::find().all(&self.db).await?;

        let mut org_code_map: HashMap<Uuid, String> = HashMap::new();
        for o in orgs {
            org_code_map.insert(o.id, o.code);
        }
        let mut org_label_map: HashMap<(Uuid, Uuid), String> = HashMap::new();
        for ol in org_labels {
            org_label_map.insert((ol.organization_id, ol.language_id), ol.label);
        }

        // Preload company labels
        let comp_labels = db_company_label::Entity::find().all(&self.db).await?;
        let mut comp_label_map: HashMap<(Uuid, Uuid), String> = HashMap::new();
        for cl in comp_labels {
            comp_label_map.insert((cl.company_id, cl.language_id), cl.label);
        }

        // Preload company types
        let type_labels = db_master_lookup_label::Entity::find()
            .all(&self.db)
            .await?;
        let mut type_label_map: HashMap<(Uuid, Uuid), String> = HashMap::new();
        for tl in type_labels {
            type_label_map.insert((tl.master_lookup_id, tl.language_id), tl.label);
        }

        let search_term = filter.search.as_deref().unwrap_or("").trim().to_lowercase();

        let mut results: Vec<CompanyDto> = Vec::new();
        for c in companies {
            let org_code = org_code_map
                .get(&c.organization_id)
                .cloned()
                .unwrap_or_default();
            let org_name_th = org_label_map
                .get(&(c.organization_id, THAI_LANGUAGE_ID))
                .cloned()
                .unwrap_or_default();
            let org_name_en = org_label_map
                .get(&(c.organization_id, ENGLISH_LANGUAGE_ID))
                .cloned()
                .unwrap_or_default();
            let org_name = pick(&org_name_th, &org_name_en).to_string();
            let name_th = comp_label_map
                .get(&(c.id, THAI_LANGUAGE_ID))
                .cloned()
                .unwrap_or_default();
            let name_en = comp_label_map
                .get(&(c.id, ENGLISH_LANGUAGE_ID))
                .cloned()
                .unwrap_or_default();
            let type_th = type_label_map
                .get(&(c.master_lookup_id, THAI_LANGUAGE_ID))
                .cloned()
                .unwrap_or_default();
            let type_en = type_label_map
                .get(&(c.master_lookup_id, ENGLISH_LANGUAGE_ID))
                .cloned()
                .unwrap_or_default();
            let company_type_name = match pick(&type_th, &type_en) {
                "" => "บริษัทจำกัด".to_string(),
                name => name.to_string(),
            };

            if !search_term.is_empty() {
                let matches = c.code.to_lowercase().contains(&search_term)
                    || name_th.to_lowercase().contains(&search_term)
                    || name_en.to_lowercase().contains(&search_term)
                    || org_code.to_lowercase().contains(&search_term)
                    || org_name_th.to_lowercase().contains(&search_term)
                    || org_name_en.to_lowercase().contains(&search_term);
                if !matches {
                    continue;
                }
            }

            results.push(CompanyDto {
                id: c.id,
                organization_id: c.organization_id,
                organization_code: org_code,
                organization_name: org_name,
                code: c.code,
                master_lookup_id: c.master_lookup_id,
                company_type_name,
                active: c.active,
                name_th,
                name_en,
                row_version: c.row_version,
            });
        }

        // Apply pagination if specified
        if let (Some(page), Some(limit)) = (filter.page, filter.limit) {
            if limit > 0 {
                let start = (page.saturating_sub(1) * limit) as usize;
                if start < results.len() {
                    let end = (start + limit as usize).min(results.len());
                    return Ok(results[start..end].to_vec());
                } else {
                    return Ok(Vec::new());
                }
            }
        }

        Ok(results)
    }
}
