//! SUPG001 lookups shared by the system and module queries: localized names, program counts and
//! the search match

use sea_orm::EntityTrait;
use std::collections::HashMap;
use uuid::Uuid;

use super::db_supg001_queries::DbSupg001Queries;
use crate::features::db::dbpg002::queries::db_dbpg002_queries::{Names, fold_names};
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{db_module_label, db_program, db_system_label};

pub(super) fn matches(term: &str, fields: &[&str]) -> bool {
    term.is_empty() || fields.iter().any(|f| f.to_lowercase().contains(term))
}

impl DbSupg001Queries {
    pub(super) async fn system_names(&self) -> Result<Names, Supg001Error> {
        Ok(fold_names(
            db_system_label::Entity::find()
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.system_id, l.language_id, l.label)),
        ))
    }

    pub(super) async fn module_names(&self) -> Result<Names, Supg001Error> {
        Ok(fold_names(
            db_module_label::Entity::find()
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.module_id, l.language_id, l.label)),
        ))
    }

    /// Program counts per system id and per module id
    pub(super) async fn program_counts(&self) -> Result<(HashMap<Uuid, u64>, HashMap<Uuid, u64>), Supg001Error> {
        let mut by_system: HashMap<Uuid, u64> = HashMap::new();
        let mut by_module: HashMap<Uuid, u64> = HashMap::new();
        for p in db_program::Entity::find().all(&self.db).await? {
            *by_system.entry(p.system_id).or_default() += 1;
            *by_module.entry(p.module_id).or_default() += 1;
        }
        Ok((by_system, by_module))
    }
}
