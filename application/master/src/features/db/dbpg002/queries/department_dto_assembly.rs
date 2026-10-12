//! DBPG002 query: assembles `DepartmentDto`s (company, branch, labels) for list and get

use sea_orm::EntityTrait;
use std::collections::HashMap;
use uuid::Uuid;

use super::label_lookup::{fold_names, matches_search};
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{
    DepartmentDto, db_branch, db_branch_label, db_department, db_department_label,
};
use crate::features::db::dbpg002::queries::db_dbpg002_queries::DbDbpg002Queries;
use crate::features::localization::request_language::pick;

impl DbDbpg002Queries {
    pub(super) async fn department_dtos(
        &self,
        departments: Vec<db_department::Model>,
        search: Option<&str>,
    ) -> Result<Vec<DepartmentDto>, Dbpg002Error> {
        let (company_codes, company_names) = self.company_info().await?;
        let branch_codes: HashMap<Uuid, String> = db_branch::Entity::find()
            .all(&self.db)
            .await?
            .into_iter()
            .map(|b| (b.id, b.code))
            .collect();
        let branch_names = fold_names(
            db_branch_label::Entity::find()
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.branch_id, l.language_id, l.label)),
        );
        let department_names = fold_names(
            db_department_label::Entity::find()
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.department_id, l.language_id, l.label)),
        );

        let term = search.unwrap_or("").trim().to_lowercase();
        let mut out = Vec::new();
        for d in departments {
            let (name_th, name_en) = department_names.get(&d.id).cloned().unwrap_or_default();
            let company_code = company_codes
                .get(&d.company_id)
                .cloned()
                .unwrap_or_default();
            let (company_th, company_en) = company_names
                .get(&d.company_id)
                .cloned()
                .unwrap_or_default();
            let company_name = pick(&company_th, &company_en).to_string();
            let branch_code = branch_codes.get(&d.branch_id).cloned().unwrap_or_default();
            let (branch_th, branch_en) = branch_names
                .get(&d.branch_id)
                .cloned()
                .unwrap_or_default();
            let branch_name = pick(&branch_th, &branch_en).to_string();
            if !matches_search(
                &term,
                &[
                    &d.code,
                    &name_th,
                    &name_en,
                    &company_code,
                    &company_th,
                    &company_en,
                    &branch_code,
                    &branch_th,
                    &branch_en,
                ],
            ) {
                continue;
            }
            out.push(DepartmentDto {
                id: d.id,
                organization_id: d.organization_id,
                company_id: d.company_id,
                company_code,
                company_name,
                branch_id: d.branch_id,
                branch_code,
                branch_name,
                code: d.code,
                active: d.active,
                name_th,
                name_en,
                row_version: d.row_version,
            });
        }
        Ok(out)
    }
}
