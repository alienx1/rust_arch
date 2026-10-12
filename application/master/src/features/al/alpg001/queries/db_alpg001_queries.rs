//! ALPG001 Sea-ORM query runner over `rp_report_job`

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::entity::rp_report_job::{Column, Entity as ReportJob};
use sea_orm::sea_query::{Expr, ExprTrait, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use super::Alpg001Queries;
use crate::features::al::alpg001::errors::Alpg001Error;
use crate::features::al::alpg001::models::{ProcessCountsDto, ProcessModel, ProcessSearch, process_status};

#[derive(Clone)]
pub struct DbAlpg001Queries {
    db: DatabaseConnection,
}

impl DbAlpg001Queries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn db_err(e: sea_orm::DbErr) -> Alpg001Error {
    Alpg001Error::Internal(e.to_string())
}

/// The filter as SQL; the status only when `with_status` (the counters leave it out)
fn condition(f: &ProcessSearch, with_status: bool) -> Condition {
    let mut c = Condition::all();
    if with_status {
        if let Some(s) = &f.status {
            c = c.add(Column::Status.eq(s.as_str()));
        }
    }
    if let Some(k) = &f.kind {
        c = c.add(Column::ReportKind.eq(k.as_str()));
    }
    if let Some(u) = &f.user {
        c = c.add(Column::RequestedBy.eq(u.as_str()));
    }
    if let Some(t) = f.from {
        c = c.add(Column::RequestedAt.gte(t.fixed_offset()));
    }
    if let Some(t) = f.to {
        c = c.add(Column::RequestedAt.lt(t.fixed_offset()));
    }
    c
}

#[async_trait]
impl Alpg001Queries for DbAlpg001Queries {
    async fn search(
        &self,
        f: &ProcessSearch,
    ) -> Result<(Vec<ProcessModel>, u64, ProcessCountsDto), Alpg001Error> {
        let last_activity: SimpleExpr = Expr::cust("COALESCE(\"finished_at\", \"started_at\", \"requested_at\")");
        let total = ReportJob::find().filter(condition(f, true)).count(&self.db).await.map_err(db_err)?;
        let items = ReportJob::find()
            .filter(condition(f, true))
            .order_by_desc(last_activity)
            .order_by_desc(Column::Id)
            .offset(f.page.saturating_sub(1) * f.limit)
            .limit(f.limit)
            .all(&self.db)
            .await
            .map_err(db_err)?;

        let grouped: Vec<(String, i64)> = ReportJob::find()
            .filter(condition(f, false))
            .select_only()
            .column(Column::Status)
            .column_as(Column::Id.count(), "n")
            .group_by(Column::Status)
            .into_tuple()
            .all(&self.db)
            .await
            .map_err(db_err)?;
        let mut counts = ProcessCountsDto::default();
        for (status, n) in grouped {
            let n = u64::try_from(n).unwrap_or(0);
            match status.as_str() {
                process_status::QUEUED => counts.queued = n,
                process_status::RUNNING => counts.running = n,
                process_status::SUCCEEDED => counts.succeeded = n,
                process_status::FAILED => counts.failed = n,
                _ => {}
            }
        }
        Ok((items, total, counts))
    }

    async fn find(&self, id: Uuid) -> Result<Option<ProcessModel>, Alpg001Error> {
        ReportJob::find_by_id(id).one(&self.db).await.map_err(db_err)
    }

    async fn mark_queued(
        &self,
        id: Uuid,
        now: DateTime<Utc>,
        stuck_before: DateTime<Utc>,
    ) -> Result<bool, Alpg001Error> {
        let allowed = Condition::any()
            .add(Column::Status.is_in([process_status::SUCCEEDED, process_status::FAILED]))
            .add(
                Condition::all()
                    .add(Column::Status.is_in([process_status::QUEUED, process_status::RUNNING]))
                    .add(Column::StartedAt.lte(stuck_before.fixed_offset())),
            );
        let done = ReportJob::update_many()
            .col_expr(Column::Status, Expr::value(process_status::QUEUED))
            .col_expr(Column::Error, Expr::value(Option::<String>::None))
            .col_expr(Column::StartedAt, Expr::value(now.fixed_offset()))
            .col_expr(Column::FinishedAt, Expr::value(Option::<sea_orm::prelude::DateTimeWithTimeZone>::None))
            .col_expr(Column::RowVersion, Expr::col(Column::RowVersion).add(1))
            .filter(Column::Id.eq(id))
            .filter(allowed)
            .exec(&self.db)
            .await
            .map_err(db_err)?;
        Ok(done.rows_affected > 0)
    }

    async fn restore(&self, row: ProcessModel) -> Result<(), Alpg001Error> {
        ReportJob::update_many()
            .col_expr(Column::Status, Expr::value(row.status))
            .col_expr(Column::Error, Expr::value(row.error))
            .col_expr(Column::StartedAt, Expr::value(row.started_at))
            .col_expr(Column::FinishedAt, Expr::value(row.finished_at))
            .col_expr(Column::RowVersion, Expr::col(Column::RowVersion).add(1))
            .filter(Column::Id.eq(row.id))
            .exec(&self.db)
            .await
            .map_err(db_err)?;
        Ok(())
    }
}
