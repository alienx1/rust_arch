//! ALPG000 program-options query: every catalogue program with a page, with its Thai / English name.

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use super::db_alpg000_queries::DbAlpg000Queries;
use crate::features::al::alpg000::errors::Alpg000Error;
use crate::features::al::alpg000::models::ProgramOptionDto;
use crate::features::db::dbpg002::queries::db_dbpg002_queries::fold_names;
use domain::entity::{db_program, db_program_label};

impl DbAlpg000Queries {
    pub(super) async fn program_options(&self) -> Result<Vec<ProgramOptionDto>, Alpg000Error> {
        let programs = db_program::Entity::find()
            .filter(db_program::Column::Path.is_not_null())
            .order_by_asc(db_program::Column::Code)
            .all(&self.db)
            .await
            .map_err(|e| Alpg000Error::Internal(e.to_string()))?;
        let names = fold_names(
            db_program_label::Entity::find()
                .all(&self.db)
                .await
                .map_err(|e| Alpg000Error::Internal(e.to_string()))?
                .into_iter()
                .map(|l| (l.program_id, l.language_id, l.label)),
        );
        Ok(programs
            .into_iter()
            .map(|p| {
                let (name_th, name_en) = names.get(&p.id).cloned().unwrap_or_default();
                ProgramOptionDto { code: p.code, name_th, name_en }
            })
            .collect())
    }
}
