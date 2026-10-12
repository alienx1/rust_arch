//! DBPG006 query: list one page of label entries matching the filter
//!
//! Paging runs over the distinct `(category, key)` pairs that have at least one matching row;
//! the translations of those pairs (all languages, matching or not) are loaded afterwards.

use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};

use crate::features::db::dbpg003::queries::db_dbpg003_queries::{contains_ci, escape_like};
use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::{
    DEFAULT_PAGE_SIZE, LabelFilterQuery, LabelPageDto, MAX_PAGE_SIZE, db_label,
};

use super::db_dbpg006_queries::DbDbpg006Queries;

impl DbDbpg006Queries {
    pub(super) async fn list_labels(
        &self,
        filter: LabelFilterQuery,
    ) -> Result<LabelPageDto, Dbpg006Error> {
        let mut cond = Condition::all();
        if let Some(category) = filter
            .category
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty())
        {
            cond = cond.add(db_label::Column::Category.eq(category));
        }

        let term = filter.search.as_deref().unwrap_or("").trim().to_lowercase();
        if !term.is_empty() {
            let like = format!("%{}%", escape_like(&term));
            cond = cond.add(
                Condition::any()
                    .add(contains_ci(db_label::Column::Category, &like))
                    .add(contains_ci(db_label::Column::Key, &like))
                    .add(contains_ci(db_label::Column::Value, &like)),
            );
        }

        let limit = Ord::clamp(filter.limit.unwrap_or(DEFAULT_PAGE_SIZE), 1, MAX_PAGE_SIZE);
        let page = Ord::max(filter.page.unwrap_or(1), 1);

        let paginator = db_label::Entity::find()
            .select_only()
            .column(db_label::Column::Category)
            .column(db_label::Column::Key)
            .filter(cond)
            .group_by(db_label::Column::Category)
            .group_by(db_label::Column::Key)
            .order_by_asc(db_label::Column::Category)
            .order_by_asc(db_label::Column::Key)
            .into_tuple::<(String, String)>()
            .paginate(&self.db, limit);
        let total = paginator.num_items().await?;
        let pairs = paginator.fetch_page(page - 1).await?;

        Ok(LabelPageDto {
            items: self.entries_for(&pairs).await?,
            total,
            page,
            limit,
        })
    }
}
