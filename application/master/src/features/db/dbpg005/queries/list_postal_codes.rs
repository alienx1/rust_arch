//! List postal codes: SQL filter / search / paging

use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use super::db_dbpg005_queries::DbDbpg005Queries;
use super::search_paging::{like_of, page_and_limit};
use crate::features::db::dbpg003::queries::db_dbpg003_queries::contains_ci;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{
    PostalFilterQuery, PostalPageDto, db_district_label, db_postal_code, db_sub_district_label,
};

impl DbDbpg005Queries {
    pub(super) async fn list_postal_codes(
        &self,
        filter: PostalFilterQuery,
    ) -> Result<PostalPageDto, Dbpg005Error> {
        let mut cond = Condition::all();
        if let Some(p) = filter.province_id {
            cond = cond.add(db_postal_code::Column::ProvinceId.eq(p));
        }
        if let Some(d) = filter.district_id {
            cond = cond.add(db_postal_code::Column::DistrictId.eq(d));
        }
        if let Some(s) = filter.sub_district_id {
            cond = cond.add(db_postal_code::Column::SubDistrictId.eq(s));
        }
        if let Some(a) = filter.active {
            cond = cond.add(db_postal_code::Column::Active.eq(a));
        }
        if let Some(like) = like_of(&filter.search) {
            let sub_ids: Vec<Uuid> = db_sub_district_label::Entity::find()
                .filter(contains_ci(db_sub_district_label::Column::Label, &like))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| l.sub_district_id)
                .collect();
            let district_ids: Vec<Uuid> = db_district_label::Entity::find()
                .filter(contains_ci(db_district_label::Column::Label, &like))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| l.district_id)
                .collect();
            cond = cond.add(
                Condition::any()
                    .add(contains_ci(db_postal_code::Column::PostalCode, &like))
                    .add(db_postal_code::Column::SubDistrictId.is_in(sub_ids))
                    .add(db_postal_code::Column::DistrictId.is_in(district_ids)),
            );
        }
        let (page, limit) = page_and_limit(filter.page, filter.limit);
        let paginator = db_postal_code::Entity::find()
            .filter(cond)
            .order_by_asc(db_postal_code::Column::PostalCode)
            .order_by_asc(db_postal_code::Column::Id)
            .paginate(&self.db, limit);
        let total = paginator.num_items().await?;
        let rows = paginator.fetch_page(page - 1).await?;
        Ok(PostalPageDto { items: self.postal_dtos(rows).await?, total, page, limit })
    }
}
