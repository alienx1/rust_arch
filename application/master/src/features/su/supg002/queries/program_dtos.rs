//! SUPG002 build `ProgramDto` rows (names, system / module / parent codes, child counts)

use sea_orm::EntityTrait;
use std::collections::HashMap;
use uuid::Uuid;

use super::db_supg002_queries::DbSupg002Queries;
use super::label_lookup::program_names;
use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::{ProgramDto, db_program, is_protected_program};
use domain::entity::{db_module, db_system};

impl DbSupg002Queries {
    pub(super) async fn program_dtos(&self, rows: Vec<db_program::Model>) -> Result<Vec<ProgramDto>, Supg002Error> {
        let names = program_names(&self.db).await?;
        let system_codes: HashMap<Uuid, String> = db_system::Entity::find()
            .all(&self.db)
            .await?
            .into_iter()
            .map(|s| (s.id, s.code))
            .collect();
        let module_codes: HashMap<Uuid, String> = db_module::Entity::find()
            .all(&self.db)
            .await?
            .into_iter()
            .map(|m| (m.id, m.code))
            .collect();
        // Parent codes and child counts come from the whole catalogue (a few dozen rows)
        let all = db_program::Entity::find().all(&self.db).await?;
        let codes: HashMap<Uuid, String> = all.iter().map(|p| (p.id, p.code.clone())).collect();
        let mut children: HashMap<Uuid, u64> = HashMap::new();
        for p in &all {
            if let Some(parent) = p.program_id {
                *children.entry(parent).or_default() += 1;
            }
        }
        Ok(rows
            .into_iter()
            .map(|p| {
                let (name_th, name_en) = names.get(&p.id).cloned().unwrap_or_default();
                ProgramDto {
                    id: p.id,
                    system_id: p.system_id,
                    system_code: system_codes.get(&p.system_id).cloned().unwrap_or_default(),
                    module_id: p.module_id,
                    module_code: module_codes.get(&p.module_id).cloned().unwrap_or_default(),
                    parent_id: p.program_id,
                    parent_code: p.program_id.and_then(|id| codes.get(&id).cloned()).unwrap_or_default(),
                    is_protected: is_protected_program(&p.code),
                    child_count: children.get(&p.id).copied().unwrap_or(0),
                    code: p.code,
                    path: p.path,
                    icon: p.icon,
                    sort_order: p.sort_order,
                    active: p.active,
                    name_th,
                    name_en,
                    row_version: p.row_version,
                }
            })
            .collect())
    }
}
