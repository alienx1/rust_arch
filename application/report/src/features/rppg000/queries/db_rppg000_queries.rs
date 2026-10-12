//! Sea-ORM implementation of the RPPG000 queries (`rp_report_job`)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::entity::rp_report_job::{ActiveModel, Column, Entity as ReportJob};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use serde_json::Value;
use uuid::Uuid;

use super::Rppg000Queries;
use crate::features::rppg000::errors::Rppg000Error;
use crate::features::rppg000::models::{ReportJobModel, job_status};

pub struct DbRppg000Queries {
    db: DatabaseConnection,
}

impl DbRppg000Queries {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn db_err(e: sea_orm::DbErr) -> Rppg000Error {
    Rppg000Error::Internal(e.to_string())
}

fn now() -> sea_orm::prelude::DateTimeWithTimeZone {
    Utc::now().fixed_offset()
}

#[async_trait]
impl Rppg000Queries for DbRppg000Queries {
    async fn start_job(
        &self,
        id: Uuid,
        report_kind: &str,
        requested_by: &str,
        filter: Value,
        requested_at: DateTime<Utc>,
    ) -> Result<(), Rppg000Error> {
        match ReportJob::find_by_id(id).one(&self.db).await.map_err(db_err)? {
            Some(existing) => {
                let mut row: ActiveModel = existing.into();
                row.status = Set(job_status::RUNNING.to_string());
                row.started_at = Set(Some(now()));
                row.finished_at = Set(None);
                row.error = Set(None);
                row.update(&self.db).await.map_err(db_err)?;
            }
            None => {
                ActiveModel {
                    id: Set(id),
                    report_kind: Set(report_kind.to_string()),
                    requested_by: Set(requested_by.to_string()),
                    filter: Set(filter),
                    status: Set(job_status::RUNNING.to_string()),
                    row_count: Set(0),
                    size_bytes: Set(0),
                    requested_at: Set(requested_at.fixed_offset()),
                    started_at: Set(Some(now())),
                    row_version: Set(1),
                    ..Default::default()
                }
                .insert(&self.db)
                .await
                .map_err(db_err)?;
            }
        }
        Ok(())
    }

    async fn find_job(&self, id: Uuid) -> Result<Option<ReportJobModel>, Rppg000Error> {
        ReportJob::find_by_id(id).one(&self.db).await.map_err(db_err)
    }

    async fn finish_job_ok(
        &self,
        id: Uuid,
        row_count: i64,
        size_bytes: i64,
        file_name: &str,
        object_key: &str,
    ) -> Result<(), Rppg000Error> {
        let row = self.find_job(id).await?.ok_or_else(|| Rppg000Error::NotFound(id.to_string()))?;
        let version = row.row_version + 1;
        let mut row: ActiveModel = row.into();
        row.status = Set(job_status::SUCCEEDED.to_string());
        row.row_count = Set(row_count);
        row.size_bytes = Set(size_bytes);
        row.file_name = Set(Some(file_name.to_string()));
        row.object_key = Set(Some(object_key.to_string()));
        row.finished_at = Set(Some(now()));
        row.row_version = Set(version);
        row.update(&self.db).await.map_err(db_err)?;
        Ok(())
    }

    async fn finish_job_failed(&self, id: Uuid, error: &str) -> Result<(), Rppg000Error> {
        let row = self.find_job(id).await?.ok_or_else(|| Rppg000Error::NotFound(id.to_string()))?;
        let version = row.row_version + 1;
        let mut row: ActiveModel = row.into();
        row.status = Set(job_status::FAILED.to_string());
        row.error = Set(Some(error.chars().take(1000).collect()));
        row.finished_at = Set(Some(now()));
        row.row_version = Set(version);
        row.update(&self.db).await.map_err(db_err)?;
        Ok(())
    }

    async fn list_jobs(&self, requested_by: &str, limit: u64) -> Result<Vec<ReportJobModel>, Rppg000Error> {
        ReportJob::find()
            .filter(Column::RequestedBy.eq(requested_by))
            .order_by_desc(Column::RequestedAt)
            .limit(limit)
            .all(&self.db)
            .await
            .map_err(db_err)
    }
}
