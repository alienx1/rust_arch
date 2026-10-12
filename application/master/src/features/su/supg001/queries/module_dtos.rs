//! SUPG001 build `ModuleDto` rows (names, system, program counts, search filter)

use sea_orm::EntityTrait;
use std::collections::HashMap;
use uuid::Uuid;

use super::db_supg001_queries::DbSupg001Queries;
use super::label_lookup::matches;
use crate::features::localization::request_language::pick;
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{ModuleDto, db_module, db_system, is_protected_module};

impl DbSupg001Queries {
    pub(super) async fn module_dtos(&self, rows: Vec<db_module::Model>, term: &str) -> Result<Vec<ModuleDto>, Supg001Error> {
        let names = self.module_names().await?;
        let system_names = self.system_names().await?;
        let system_codes: HashMap<Uuid, String> = db_system::Entity::find()
            .all(&self.db)
            .await?
            .into_iter()
            .map(|s| (s.id, s.code))
            .collect();
        let (_, program_by_module) = self.program_counts().await?;
        Ok(rows
            .into_iter()
            .filter_map(|m| {
                let (name_th, name_en) = names.get(&m.id).cloned().unwrap_or_default();
                let system_code = system_codes.get(&m.system_id).cloned().unwrap_or_default();
                matches(term, &[&m.code, &name_th, &name_en, &system_code]).then(|| ModuleDto {
                    id: m.id,
                    system_id: m.system_id,
                    system_name: system_names.get(&m.system_id).map(|n| pick(&n.0, &n.1).to_string()).unwrap_or_default(),
                    system_code,
                    is_protected: is_protected_module(&m.code),
                    program_count: program_by_module.get(&m.id).copied().unwrap_or(0),
                    code: m.code,
                    active: m.active,
                    name_th,
                    name_en,
                    row_version: m.row_version,
                })
            })
            .collect())
    }
}
