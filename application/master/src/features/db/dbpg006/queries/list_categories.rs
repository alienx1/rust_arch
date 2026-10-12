//! DBPG006 query: label categories with the number of entries (distinct keys)

use sea_orm::{EntityTrait, QuerySelect};
use std::collections::BTreeMap;

use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::{LabelCategoryDto, db_label};

use super::db_dbpg006_queries::DbDbpg006Queries;

impl DbDbpg006Queries {
    pub(super) async fn list_categories(&self) -> Result<Vec<LabelCategoryDto>, Dbpg006Error> {
        let pairs: Vec<(String, String)> = db_label::Entity::find()
            .select_only()
            .column(db_label::Column::Category)
            .column(db_label::Column::Key)
            .distinct()
            .into_tuple()
            .all(&self.db)
            .await?;
        let mut counts: BTreeMap<String, u64> = BTreeMap::new();
        for (category, _) in pairs {
            *counts.entry(category).or_default() += 1;
        }
        Ok(counts
            .into_iter()
            .map(|(category, entries)| LabelCategoryDto { category, entries })
            .collect())
    }
}
