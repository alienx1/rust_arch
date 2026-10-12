//! DBPG002 query: branch types (HEAD / BRANCH) from `db_master_lookup`

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use super::label_lookup::fold_names;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::BranchTypeLookupDto;
use crate::features::db::dbpg002::queries::db_dbpg002_queries::DbDbpg002Queries;
use domain::entity::{db_master_lookup, db_master_lookup_label};

impl DbDbpg002Queries {
    pub(super) async fn list_branch_types(&self) -> Result<Vec<BranchTypeLookupDto>, Dbpg002Error> {
        let lookups = db_master_lookup::Entity::find()
            .filter(db_master_lookup::Column::Category.eq("branch_type"))
            .filter(db_master_lookup::Column::Active.eq(true))
            .order_by_asc(db_master_lookup::Column::Code)
            .all(&self.db)
            .await?;
        let names = fold_names(
            db_master_lookup_label::Entity::find()
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.master_lookup_id, l.language_id, l.label)),
        );
        Ok(lookups
            .into_iter()
            .map(|l| {
                let (th, en) = names.get(&l.id).cloned().unwrap_or_default();
                BranchTypeLookupDto {
                    name_th: if th.is_empty() { l.code.clone() } else { th },
                    name_en: if en.is_empty() { l.code.clone() } else { en },
                    id: l.id,
                    code: l.code,
                }
            })
            .collect())
    }
}
