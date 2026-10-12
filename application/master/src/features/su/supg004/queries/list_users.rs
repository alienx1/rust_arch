//! SUPG004 list-users query: filter, search and paging in SQL, names loaded for the page only.

use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use uuid::Uuid;

use super::db_supg004_queries::DbSupg004Queries;
use crate::features::db::dbpg003::queries::db_dbpg003_queries::{contains_ci, escape_like};
use crate::features::su::supg004::errors::Supg004Error;
use crate::features::su::supg004::models::{
    DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE, UserFilterQuery, UserPageDto, su_user,
};
use domain::entity::{db_employee, db_person_name_label, su_user_assignment};

impl DbSupg004Queries {
    pub(super) async fn list_users(&self, filter: UserFilterQuery) -> Result<UserPageDto, Supg004Error> {
        let mut cond = Condition::all();
        if let Some(g) = filter.group_id {
            // Holding the group in any assignment counts, not only as the primary group
            let assigned: Vec<Uuid> = su_user_assignment::Entity::find()
                .filter(su_user_assignment::Column::PermissionGroupId.eq(g))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|a| a.user_id)
                .collect();
            cond = cond.add(
                Condition::any()
                    .add(su_user::Column::PermissionGroupId.eq(g))
                    .add(su_user::Column::Id.is_in(assigned)),
            );
        }
        if let Some(a) = filter.active {
            cond = cond.add(su_user::Column::Active.eq(a));
        }
        let term = filter.search.as_deref().unwrap_or("").trim().to_lowercase();
        if !term.is_empty() {
            let like = format!("%{}%", escape_like(&term));
            // Employees whose code or name matches
            let name_person_ids: Vec<Uuid> = db_person_name_label::Entity::find()
                .filter(
                    Condition::any()
                        .add(contains_ci(db_person_name_label::Column::FirstName, &like))
                        .add(contains_ci(db_person_name_label::Column::LastName, &like))
                        .add(contains_ci(db_person_name_label::Column::Nickname, &like)),
                )
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| l.person_name_id)
                .collect();
            let employee_ids: Vec<Uuid> = db_employee::Entity::find()
                .filter(
                    Condition::any()
                        .add(contains_ci(db_employee::Column::Code, &like))
                        .add(db_employee::Column::PersonNameId.is_in(name_person_ids)),
                )
                .all(&self.db)
                .await?
                .into_iter()
                .map(|e| e.id)
                .collect();
            cond = cond.add(
                Condition::any()
                    .add(contains_ci(su_user::Column::Username, &like))
                    .add(su_user::Column::EmployeeId.is_in(employee_ids)),
            );
        }

        let limit = Ord::clamp(filter.limit.unwrap_or(DEFAULT_PAGE_SIZE), 1, MAX_PAGE_SIZE);
        let page = Ord::max(filter.page.unwrap_or(1), 1);
        let paginator = su_user::Entity::find()
            .filter(cond)
            .order_by_asc(su_user::Column::Username)
            .paginate(&self.db, limit);
        let total = paginator.num_items().await?;
        let rows = paginator.fetch_page(page - 1).await?;
        Ok(UserPageDto { items: self.user_dtos(rows).await?, total, page, limit })
    }
}
