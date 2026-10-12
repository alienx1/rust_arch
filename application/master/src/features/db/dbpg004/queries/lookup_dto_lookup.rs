//! DBPG004 shared lookup: attach Thai / English labels to lookup rows

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::features::db::dbpg002::queries::db_dbpg002_queries::fold_names;
use crate::features::db::dbpg004::errors::Dbpg004Error;
use crate::features::db::dbpg004::models::{
    LookupDto, db_master_lookup, db_master_lookup_label, is_protected_category,
};

use super::db_dbpg004_queries::DbDbpg004Queries;

impl DbDbpg004Queries {
    pub(super) async fn lookup_dtos(
        &self,
        rows: Vec<db_master_lookup::Model>,
    ) -> Result<Vec<LookupDto>, Dbpg004Error> {
        if rows.is_empty() {
            return Ok(Vec::new());
        }
        let ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
        let names = fold_names(
            db_master_lookup_label::Entity::find()
                .filter(db_master_lookup_label::Column::MasterLookupId.is_in(ids))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.master_lookup_id, l.language_id, l.label)),
        );
        Ok(rows
            .into_iter()
            .map(|r| {
                let (name_th, name_en) = names.get(&r.id).cloned().unwrap_or_default();
                LookupDto {
                    id: r.id,
                    is_protected: is_protected_category(&r.category),
                    category: r.category,
                    code: r.code,
                    active: r.active,
                    name_th,
                    name_en,
                    row_version: r.row_version,
                }
            })
            .collect())
    }
}
