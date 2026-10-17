//! DBPG003 query: all branches (filtered client-side by company)

use sea_orm::{EntityTrait, QueryOrder};

use crate::features::db::dbpg002::models::{db_branch, db_branch_label};
use crate::features::db::dbpg002::queries::db_dbpg002_queries::fold_names;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::BranchOptionDto;
use crate::features::db::dbpg003::queries::db_dbpg003_queries::DbDbpg003Queries;
use crate::features::db::view_scope_filters::{by_branch_rows, by_company};

impl DbDbpg003Queries {
    pub(super) async fn list_branch_options(&self) -> Result<Vec<BranchOptionDto>, Dbpg003Error> {
        let branches = by_branch_rows(by_company(db_branch::Entity::find(), db_branch::Column::CompanyId), db_branch::Column::Id)
            .order_by_asc(db_branch::Column::Code)
            .all(&self.db)
            .await?;
        let names = fold_names(
            db_branch_label::Entity::find()
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.branch_id, l.language_id, l.label)),
        );
        Ok(branches
            .into_iter()
            .map(|b| {
                let (name_th, name_en) = names.get(&b.id).cloned().unwrap_or_default();
                BranchOptionDto {
                    id: b.id,
                    company_id: b.company_id,
                    code: b.code,
                    name_th,
                    name_en,
                    active: b.active,
                }
            })
            .collect())
    }
}
