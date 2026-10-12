//! `SeaORM` Entity for `rp_report_job` (a report request and its outcome)

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(schema_name = "arch", table_name = "rp_report_job")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub report_kind: String,
    pub requested_by: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub filter: Json,
    pub status: String,
    pub row_count: i64,
    pub size_bytes: i64,
    pub file_name: Option<String>,
    pub object_key: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub error: Option<String>,
    pub requested_at: DateTimeWithTimeZone,
    pub started_at: Option<DateTimeWithTimeZone>,
    pub finished_at: Option<DateTimeWithTimeZone>,
    pub row_version: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
