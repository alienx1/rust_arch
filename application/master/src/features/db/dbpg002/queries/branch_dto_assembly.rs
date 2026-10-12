//! DBPG002 query: assembles `BranchDto`s (company, type, labels, department counts) for list and get

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;
use uuid::Uuid;

use super::label_lookup::{fold_names, matches_search};
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{
    BRANCH_TYPE_HEAD, BranchDto, db_branch, db_branch_label, db_department,
};
use crate::features::db::dbpg002::queries::db_dbpg002_queries::DbDbpg002Queries;
use crate::features::localization::request_language::pick;
use domain::entity::{db_master_lookup, db_master_lookup_label};

impl DbDbpg002Queries {
    pub(super) async fn branch_dtos(
        &self,
        branches: Vec<db_branch::Model>,
        search: Option<&str>,
    ) -> Result<Vec<BranchDto>, Dbpg002Error> {
        let (company_codes, company_names) = self.company_info().await?;
        let branch_names = fold_names(
            db_branch_label::Entity::find()
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.branch_id, l.language_id, l.label)),
        );
        let type_codes: HashMap<Uuid, String> = db_master_lookup::Entity::find()
            .filter(db_master_lookup::Column::Category.eq("branch_type"))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|l| (l.id, l.code))
            .collect();
        let type_names = fold_names(
            db_master_lookup_label::Entity::find()
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.master_lookup_id, l.language_id, l.label)),
        );
        let mut department_counts: HashMap<Uuid, u64> = HashMap::new();
        for d in db_department::Entity::find().all(&self.db).await? {
            *department_counts.entry(d.branch_id).or_default() += 1;
        }

        let term = search.unwrap_or("").trim().to_lowercase();
        let mut out = Vec::new();
        for b in branches {
            let (name_th, name_en) = branch_names.get(&b.id).cloned().unwrap_or_default();
            let company_code = company_codes
                .get(&b.company_id)
                .cloned()
                .unwrap_or_default();
            let (company_th, company_en) = company_names
                .get(&b.company_id)
                .cloned()
                .unwrap_or_default();
            let company_name = pick(&company_th, &company_en).to_string();
            if !matches_search(
                &term,
                &[&b.code, &name_th, &name_en, &company_code, &company_th, &company_en],
            ) {
                continue;
            }
            let branch_type_code = type_codes
                .get(&b.master_lookup_id)
                .cloned()
                .unwrap_or_default();
            let (type_th, type_en) = type_names
                .get(&b.master_lookup_id)
                .cloned()
                .unwrap_or_default();
            let branch_type_name = match pick(&type_th, &type_en) {
                "" => branch_type_code.clone(),
                name => name.to_string(),
            };
            out.push(BranchDto {
                id: b.id,
                organization_id: b.organization_id,
                company_id: b.company_id,
                company_code,
                company_name,
                code: b.code,
                master_lookup_id: b.master_lookup_id,
                is_headquarter: branch_type_code == BRANCH_TYPE_HEAD,
                branch_type_code,
                branch_type_name,
                active: b.active,
                name_th,
                name_en,
                department_count: department_counts.get(&b.id).copied().unwrap_or(0),
                row_version: b.row_version,
            });
        }
        Ok(out)
    }
}
