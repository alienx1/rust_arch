//! Sea-ORM implementation of the NTPG000 queries (`nt_notification`)

use async_trait::async_trait;
use chrono::Utc;
use domain::entity::nt_notification::{ActiveModel, Column, Entity as Notification};
use sea_orm::sea_query::{Expr, ExprTrait, OnConflict};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use super::Ntpg000Queries;
use crate::features::ntpg000::errors::Ntpg000Error;
use crate::features::ntpg000::models::NotificationModel;

pub struct DbNtpg000Queries {
    db: DatabaseConnection,
}

impl DbNtpg000Queries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn db_err(e: sea_orm::DbErr) -> Ntpg000Error {
    Ntpg000Error::Internal(e.to_string())
}

#[async_trait]
impl Ntpg000Queries for DbNtpg000Queries {
    async fn insert_if_absent(&self, row: NotificationModel) -> Result<bool, Ntpg000Error> {
        let active: ActiveModel = row.into();
        let inserted = Notification::insert(active)
            .on_conflict(OnConflict::column(Column::Id).do_nothing().to_owned())
            .exec_without_returning(&self.db)
            .await
            .map_err(db_err)?;
        Ok(inserted > 0)
    }

    async fn list(&self, recipient: &str, limit: u64) -> Result<Vec<NotificationModel>, Ntpg000Error> {
        Notification::find()
            .filter(Column::Recipient.eq(recipient))
            .order_by_desc(Column::CreatedAt)
            .limit(limit)
            .all(&self.db)
            .await
            .map_err(db_err)
    }

    async fn prune(&self, recipient: &str, keep: u64) -> Result<u64, Ntpg000Error> {
        let stale: Vec<Uuid> = Notification::find()
            .select_only()
            .column(Column::Id)
            .filter(Column::Recipient.eq(recipient))
            .order_by_desc(Column::CreatedAt)
            .order_by_desc(Column::Id)
            .offset(keep)
            .into_tuple()
            .all(&self.db)
            .await
            .map_err(db_err)?;
        if stale.is_empty() {
            return Ok(0);
        }
        let done = Notification::delete_many()
            .filter(Column::Recipient.eq(recipient))
            .filter(Column::Id.is_in(stale))
            .exec(&self.db)
            .await
            .map_err(db_err)?;
        Ok(done.rows_affected)
    }

    async fn count_unread(&self, recipient: &str) -> Result<u64, Ntpg000Error> {
        Notification::find()
            .filter(Column::Recipient.eq(recipient))
            .filter(Column::ReadAt.is_null())
            .count(&self.db)
            .await
            .map_err(db_err)
    }

    async fn mark_read(&self, recipient: &str, id: Uuid) -> Result<bool, Ntpg000Error> {
        let done = Notification::update_many()
            .col_expr(Column::ReadAt, Expr::value(Utc::now().fixed_offset()))
            .col_expr(Column::RowVersion, Expr::col(Column::RowVersion).add(1))
            .filter(Column::Id.eq(id))
            .filter(Column::Recipient.eq(recipient))
            .filter(Column::ReadAt.is_null())
            .exec(&self.db)
            .await
            .map_err(db_err)?;
        Ok(done.rows_affected > 0)
    }

    async fn mark_all_read(&self, recipient: &str) -> Result<u64, Ntpg000Error> {
        let done = Notification::update_many()
            .col_expr(Column::ReadAt, Expr::value(Utc::now().fixed_offset()))
            .col_expr(Column::RowVersion, Expr::col(Column::RowVersion).add(1))
            .filter(Column::Recipient.eq(recipient))
            .filter(Column::ReadAt.is_null())
            .exec(&self.db)
            .await
            .map_err(db_err)?;
        Ok(done.rows_affected)
    }
}
