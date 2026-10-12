//! DBPG006 query: active languages with Thai / English display names

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use std::collections::HashMap;
use uuid::Uuid;

use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::{
    ENGLISH_LANGUAGE_ID, LanguageOptionDto, THAI_LANGUAGE_ID, db_language, db_language_label,
};

use super::db_dbpg006_queries::DbDbpg006Queries;

impl DbDbpg006Queries {
    /// Thai first, then by code; the name falls back to the code when no label exists
    pub(super) async fn list_languages(&self) -> Result<Vec<LanguageOptionDto>, Dbpg006Error> {
        let languages = db_language::Entity::find()
            .filter(db_language::Column::Active.eq(true))
            .order_by_asc(db_language::Column::Code)
            .all(&self.db)
            .await?;
        let labels = db_language_label::Entity::find().all(&self.db).await?;

        // (language, display language) -> label
        let names: HashMap<(Uuid, Uuid), String> = labels
            .into_iter()
            .map(|l| ((l.language_id, l.display_language_id), l.label))
            .collect();

        let mut options: Vec<LanguageOptionDto> = languages
            .into_iter()
            .map(|l| LanguageOptionDto {
                name_th: names
                    .get(&(l.id, THAI_LANGUAGE_ID))
                    .cloned()
                    .unwrap_or_else(|| l.code.clone()),
                name_en: names
                    .get(&(l.id, ENGLISH_LANGUAGE_ID))
                    .cloned()
                    .unwrap_or_else(|| l.code.clone()),
                id: l.id,
                code: l.code,
            })
            .collect();
        options.sort_by_key(|o| o.id != THAI_LANGUAGE_ID);
        Ok(options)
    }
}
