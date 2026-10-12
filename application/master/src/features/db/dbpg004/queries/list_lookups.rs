//! DBPG004 query: list one page of lookups matching the filter

use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::features::db::dbpg003::queries::db_dbpg003_queries::{contains_ci, escape_like};
use crate::features::db::dbpg004::errors::Dbpg004Error;
use crate::features::db::dbpg004::models::{
    DEFAULT_PAGE_SIZE, LookupFilterQuery, LookupPageDto, MAX_PAGE_SIZE, db_master_lookup,
    db_master_lookup_label,
};

use super::db_dbpg004_queries::DbDbpg004Queries;

impl DbDbpg004Queries {
    pub(super) async fn list_lookups(
        &self,
        filter: LookupFilterQuery,
    ) -> Result<LookupPageDto, Dbpg004Error> {
        let mut cond = Condition::all();
        if let Some(category) = filter
            .category
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty())
        {
            cond = cond.add(db_master_lookup::Column::Category.eq(category));
        }
        if let Some(active) = filter.active {
            cond = cond.add(db_master_lookup::Column::Active.eq(active));
        }

        let term = filter.search.as_deref().unwrap_or("").trim().to_lowercase();
        if !term.is_empty() {
            let like = format!("%{}%", escape_like(&term));
            let label_ids: Vec<Uuid> = db_master_lookup_label::Entity::find()
                .filter(contains_ci(db_master_lookup_label::Column::Label, &like))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| l.master_lookup_id)
                .collect();
            cond = cond.add(
                Condition::any()
                    .add(contains_ci(db_master_lookup::Column::Code, &like))
                    .add(db_master_lookup::Column::Id.is_in(label_ids)),
            );
        }

        let limit = Ord::clamp(filter.limit.unwrap_or(DEFAULT_PAGE_SIZE), 1, MAX_PAGE_SIZE);
        let page = Ord::max(filter.page.unwrap_or(1), 1);

        let paginator = db_master_lookup::Entity::find()
            .filter(cond)
            .order_by_asc(db_master_lookup::Column::Category)
            .order_by_asc(db_master_lookup::Column::Code)
            .paginate(&self.db, limit);
        let total = paginator.num_items().await?;
        let rows = paginator.fetch_page(page - 1).await?;

        Ok(LookupPageDto {
            items: self.lookup_dtos(rows).await?,
            total,
            page,
            limit,
        })
    }
}
