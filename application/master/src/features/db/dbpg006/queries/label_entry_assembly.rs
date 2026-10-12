//! DBPG006 entry assembly: loads the translations of a set of `(category, key)` pairs and groups
//! them into entries in the order of the pairs

use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};
use std::collections::HashMap;
use uuid::Uuid;

use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::{
    LabelEntryDto, LabelValueDto, THAI_LANGUAGE_ID, db_label, db_language,
};

use super::db_dbpg006_queries::DbDbpg006Queries;

impl DbDbpg006Queries {
    pub(super) async fn entries_for(
        &self,
        pairs: &[(String, String)],
    ) -> Result<Vec<LabelEntryDto>, Dbpg006Error> {
        if pairs.is_empty() {
            return Ok(Vec::new());
        }

        let mut any = Condition::any();
        for (category, key) in pairs {
            any = any.add(
                Condition::all()
                    .add(db_label::Column::Category.eq(category.as_str()))
                    .add(db_label::Column::Key.eq(key.as_str())),
            );
        }
        let rows = db_label::Entity::find().filter(any).all(&self.db).await?;
        let codes: HashMap<Uuid, String> = db_language::Entity::find()
            .all(&self.db)
            .await?
            .into_iter()
            .map(|l| (l.id, l.code))
            .collect();

        let mut grouped: HashMap<(String, String), Vec<LabelValueDto>> = HashMap::new();
        for row in rows {
            grouped
                .entry((row.category, row.key))
                .or_default()
                .push(LabelValueDto {
                    language_code: codes
                        .get(&row.language_id)
                        .cloned()
                        .unwrap_or_else(|| row.language_id.to_string()),
                    language_id: row.language_id,
                    value: row.value,
                });
        }

        Ok(pairs
            .iter()
            .filter_map(|(category, key)| {
                let mut values = grouped.remove(&(category.clone(), key.clone()))?;
                // Thai first, then by language code
                values.sort_by(|a, b| {
                    (a.language_id != THAI_LANGUAGE_ID, &a.language_code)
                        .cmp(&(b.language_id != THAI_LANGUAGE_ID, &b.language_code))
                });
                Some(LabelEntryDto { category: category.clone(), key: key.clone(), values })
            })
            .collect())
    }
}
