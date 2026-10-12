//! List provinces: SQL filter / search / paging

use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use super::db_dbpg005_queries::DbDbpg005Queries;
use super::search_paging::{like_of, page_and_limit};
use crate::features::db::dbpg003::queries::db_dbpg003_queries::contains_ci;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{GeoFilterQuery, GeoPageDto, db_province, db_province_label};

impl DbDbpg005Queries {
    pub(super) async fn list_provinces(&self, filter: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error> {
        let mut cond = Condition::all();
        if let Some(a) = filter.active {
            cond = cond.add(db_province::Column::Active.eq(a));
        }
        if let Some(like) = like_of(&filter.search) {
            let ids: Vec<Uuid> = db_province_label::Entity::find()
                .filter(contains_ci(db_province_label::Column::Label, &like))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| l.province_id)
                .collect();
            cond = cond.add(
                Condition::any()
                    .add(contains_ci(db_province::Column::Code, &like))
                    .add(db_province::Column::Id.is_in(ids)),
            );
        }
        let (page, limit) = page_and_limit(filter.page, filter.limit);
        let paginator = db_province::Entity::find()
            .filter(cond)
            .order_by_asc(db_province::Column::Code)
            .paginate(&self.db, limit);
        let total = paginator.num_items().await?;
        let rows = paginator.fetch_page(page - 1).await?;
        Ok(GeoPageDto { items: self.province_nodes(rows).await?, total, page, limit })
    }
}
