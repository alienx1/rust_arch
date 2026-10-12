//! SUPG002 list programs: filters, label-aware search and paging

use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use super::db_supg002_queries::DbSupg002Queries;
use crate::features::db::dbpg003::queries::db_dbpg003_queries::{contains_ci, escape_like};
use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::{
    DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE, ProgramFilterQuery, ProgramPageDto, db_program,
    db_program_label,
};

impl DbSupg002Queries {
    pub(super) async fn list_programs(&self, filter: ProgramFilterQuery) -> Result<ProgramPageDto, Supg002Error> {
        let mut cond = Condition::all();
        if let Some(v) = filter.system_id {
            cond = cond.add(db_program::Column::SystemId.eq(v));
        }
        if let Some(v) = filter.module_id {
            cond = cond.add(db_program::Column::ModuleId.eq(v));
        }
        if let Some(v) = filter.active {
            cond = cond.add(db_program::Column::Active.eq(v));
        }
        let term = filter.search.as_deref().unwrap_or("").trim().to_lowercase();
        if !term.is_empty() {
            let like = format!("%{}%", escape_like(&term));
            let label_ids: Vec<Uuid> = db_program_label::Entity::find()
                .filter(contains_ci(db_program_label::Column::Label, &like))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| l.program_id)
                .collect();
            cond = cond.add(
                Condition::any()
                    .add(contains_ci(db_program::Column::Code, &like))
                    .add(contains_ci(db_program::Column::Path, &like))
                    .add(db_program::Column::Id.is_in(label_ids)),
            );
        }

        let limit = Ord::clamp(filter.limit.unwrap_or(DEFAULT_PAGE_SIZE), 1, MAX_PAGE_SIZE);
        let page = Ord::max(filter.page.unwrap_or(1), 1);
        let paginator = db_program::Entity::find()
            .filter(cond)
            .order_by_asc(db_program::Column::SystemId)
            .order_by_asc(db_program::Column::ModuleId)
            .order_by_asc(db_program::Column::SortOrder)
            .order_by_asc(db_program::Column::Code)
            .paginate(&self.db, limit);
        let total = paginator.num_items().await?;
        let rows = paginator.fetch_page(page - 1).await?;
        Ok(ProgramPageDto { items: self.program_dtos(rows).await?, total, page, limit })
    }
}
