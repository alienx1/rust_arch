//! DBPG002 query: companies for the company filter / parent selector

use sea_orm::{EntityTrait, QueryOrder};

use super::label_lookup::fold_names;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{CompanyOptionDto, db_company, db_company_label};
use crate::features::db::dbpg002::queries::db_dbpg002_queries::DbDbpg002Queries;
use crate::features::db::view_scope_filters::by_company;

impl DbDbpg002Queries {
    pub(super) async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg002Error> {
        let companies = by_company(db_company::Entity::find(), db_company::Column::Id)
            .order_by_asc(db_company::Column::Code)
            .all(&self.db)
            .await?;
        let names = fold_names(
            db_company_label::Entity::find()
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.company_id, l.language_id, l.label)),
        );
        Ok(companies
            .into_iter()
            .map(|c| {
                let (name_th, name_en) = names.get(&c.id).cloned().unwrap_or_default();
                CompanyOptionDto {
                    id: c.id,
                    organization_id: c.organization_id,
                    code: c.code,
                    name_th,
                    name_en,
                }
            })
            .collect())
    }
}
