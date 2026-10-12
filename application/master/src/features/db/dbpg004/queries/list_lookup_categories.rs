//! DBPG004 query: list lookup categories with row counts

use sea_orm::{EntityTrait, QuerySelect};
use std::collections::BTreeMap;

use crate::features::db::dbpg004::errors::Dbpg004Error;
use crate::features::db::dbpg004::models::{
    LookupCategoryDto, db_master_lookup, is_protected_category,
};

use super::db_dbpg004_queries::DbDbpg004Queries;

impl DbDbpg004Queries {
    pub(super) async fn list_categories(&self) -> Result<Vec<LookupCategoryDto>, Dbpg004Error> {
        let rows: Vec<(String, bool)> = db_master_lookup::Entity::find()
            .select_only()
            .column(db_master_lookup::Column::Category)
            .column(db_master_lookup::Column::Active)
            .into_tuple()
            .all(&self.db)
            .await?;
        let mut counts: BTreeMap<String, (u64, u64)> = BTreeMap::new();
        for (category, active) in rows {
            let entry = counts.entry(category).or_default();
            entry.0 += 1;
            entry.1 += u64::from(active);
        }
        Ok(counts
            .into_iter()
            .map(|(category, (total, active))| LookupCategoryDto {
                is_protected: is_protected_category(&category),
                category,
                total,
                active,
            })
            .collect())
    }
}
