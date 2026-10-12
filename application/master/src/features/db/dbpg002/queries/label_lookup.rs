//! DBPG002 label lookup helpers: Thai/English name folding, company info and search matching

use sea_orm::EntityTrait;
use std::collections::HashMap;
use uuid::Uuid;

use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{
    ENGLISH_LANGUAGE_ID, THAI_LANGUAGE_ID, db_company, db_company_label,
};
use crate::features::db::dbpg002::queries::db_dbpg002_queries::DbDbpg002Queries;

/// (Thai, English) names keyed by owner id
pub(crate) type Names = HashMap<Uuid, (String, String)>;

pub(crate) fn fold_names(rows: impl Iterator<Item = (Uuid, Uuid, String)>) -> Names {
    let mut map = Names::new();
    for (owner, language, label) in rows {
        let entry = map.entry(owner).or_default();
        if language == THAI_LANGUAGE_ID {
            entry.0 = label;
        } else if language == ENGLISH_LANGUAGE_ID {
            entry.1 = label;
        }
    }
    map
}

pub(super) fn matches_search(term: &str, fields: &[&str]) -> bool {
    term.is_empty() || fields.iter().any(|f| f.to_lowercase().contains(term))
}

impl DbDbpg002Queries {
    /// Company code and names by company id
    pub(super) async fn company_info(
        &self,
    ) -> Result<(HashMap<Uuid, String>, Names), Dbpg002Error> {
        let codes = db_company::Entity::find()
            .all(&self.db)
            .await?
            .into_iter()
            .map(|c| (c.id, c.code))
            .collect();
        let names = fold_names(
            db_company_label::Entity::find()
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.company_id, l.language_id, l.label)),
        );
        Ok((codes, names))
    }
}
