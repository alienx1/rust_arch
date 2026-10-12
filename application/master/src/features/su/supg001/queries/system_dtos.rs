//! SUPG001 build `SystemDto` rows (names, module / program counts, search filter)

use sea_orm::EntityTrait;
use std::collections::HashMap;
use uuid::Uuid;

use super::db_supg001_queries::DbSupg001Queries;
use super::label_lookup::matches;
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{SystemDto, db_module, db_system, is_protected_system};

impl DbSupg001Queries {
    pub(super) async fn system_dtos(&self, rows: Vec<db_system::Model>, term: &str) -> Result<Vec<SystemDto>, Supg001Error> {
        let names = self.system_names().await?;
        let (program_by_system, _) = self.program_counts().await?;
        let mut module_by_system: HashMap<Uuid, u64> = HashMap::new();
        for m in db_module::Entity::find().all(&self.db).await? {
            *module_by_system.entry(m.system_id).or_default() += 1;
        }
        Ok(rows
            .into_iter()
            .filter_map(|s| {
                let (name_th, name_en) = names.get(&s.id).cloned().unwrap_or_default();
                matches(term, &[&s.code, &name_th, &name_en]).then(|| SystemDto {
                    id: s.id,
                    is_protected: is_protected_system(&s.code),
                    module_count: module_by_system.get(&s.id).copied().unwrap_or(0),
                    program_count: program_by_system.get(&s.id).copied().unwrap_or(0),
                    code: s.code,
                    active: s.active,
                    name_th,
                    name_en,
                    row_version: s.row_version,
                })
            })
            .collect())
    }
}
