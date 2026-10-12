//! DBPG003 query: name prefixes from `db_master_lookup` (category `prefix`)

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::features::db::dbpg002::queries::db_dbpg002_queries::fold_names;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::PrefixOptionDto;
use crate::features::db::dbpg003::queries::db_dbpg003_queries::DbDbpg003Queries;
use domain::entity::{db_master_lookup, db_master_lookup_label};

impl DbDbpg003Queries {
    pub(super) async fn list_prefixes(&self) -> Result<Vec<PrefixOptionDto>, Dbpg003Error> {
        let lookups = db_master_lookup::Entity::find()
            .filter(db_master_lookup::Column::Category.eq("prefix"))
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
                PrefixOptionDto {
                    name_th: if th.is_empty() { l.code.clone() } else { th },
                    name_en: if en.is_empty() { l.code.clone() } else { en },
                    id: l.id,
                    code: l.code,
                }
            })
            .collect())
    }
}
