//! SUPG002 systems, modules and programs for the form selectors

use sea_orm::{EntityTrait, QueryOrder};

use super::db_supg002_queries::DbSupg002Queries;
use super::label_lookup::program_names;
use crate::features::db::dbpg002::queries::db_dbpg002_queries::fold_names;
use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::{
    ModuleOptionDto, ProgramOptionDto, ProgramOptionsDto, SystemOptionDto, db_program,
};
use domain::entity::{db_module, db_module_label, db_system, db_system_label};

impl DbSupg002Queries {
    pub(super) async fn program_options(&self) -> Result<ProgramOptionsDto, Supg002Error> {
        let system_names = fold_names(
            db_system_label::Entity::find()
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.system_id, l.language_id, l.label)),
        );
        let module_names = fold_names(
            db_module_label::Entity::find()
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.module_id, l.language_id, l.label)),
        );
        let program_names = program_names(&self.db).await?;

        let systems = db_system::Entity::find()
            .order_by_asc(db_system::Column::Code)
            .all(&self.db)
            .await?
            .into_iter()
            .map(|s| {
                let (name_th, name_en) = system_names.get(&s.id).cloned().unwrap_or_default();
                SystemOptionDto { id: s.id, code: s.code, name_th, name_en }
            })
            .collect();
        let modules = db_module::Entity::find()
            .order_by_asc(db_module::Column::Code)
            .all(&self.db)
            .await?
            .into_iter()
            .map(|m| {
                let (name_th, name_en) = module_names.get(&m.id).cloned().unwrap_or_default();
                ModuleOptionDto { id: m.id, system_id: m.system_id, code: m.code, name_th, name_en }
            })
            .collect();
        let programs = db_program::Entity::find()
            .order_by_asc(db_program::Column::Code)
            .all(&self.db)
            .await?
            .into_iter()
            .map(|p| {
                let (name_th, name_en) = program_names.get(&p.id).cloned().unwrap_or_default();
                ProgramOptionDto {
                    id: p.id,
                    system_id: p.system_id,
                    module_id: p.module_id,
                    parent_id: p.program_id,
                    code: p.code,
                    name_th,
                    name_en,
                }
            })
            .collect();
        Ok(ProgramOptionsDto { systems, modules, programs })
    }
}
